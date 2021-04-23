use std::borrow::{Borrow, BorrowMut};

use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyActionCondition { pub(crate) window_class_name: Option<String> }

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) enum ValueType {
    Bool(bool),
    String(String),
}

#[derive(Debug)]
pub(crate) struct VarMap {
    pub(crate) scope_values: HashMap<String, ValueType>,
    pub(crate) parent: Option<GuardedVarMap>,
}

impl PartialEq for VarMap {
    fn eq(&self, other: &Self) -> bool {
        self.scope_values == other.scope_values &&
            match (&self.parent, &other.parent) {
                (None, None) => true,
                (Some(l), Some(r)) => arc_mutexes_are_equal(&*l, &*r),
                (_, _) => false,
            }
    }
}

pub(crate) type GuardedVarMap = Arc<Mutex<VarMap>>;


#[async_recursion]
pub(crate) async fn eval_conditional_block(condition: &KeyActionCondition, block: &Block, amb: &mut Ambient) {
    // check condition
    if let Some(window_class_name) = &condition.window_class_name {
        let (tx, mut rx) = tokio::sync::mpsc::channel(0);
        amb.message_tx.as_ref().unwrap().send(ExecutionMessage::GetFocusedWindowInfo(tx)).await.unwrap();

        if let Some(active_window) = rx.recv().await.unwrap() {
            if *window_class_name != active_window.class {
                return;
            }
        } else {
            return;
        }
    }

    eval_block(block, amb).await;
}

#[async_recursion]
pub(crate) async fn eval_expr<'a>(expr: &Expr, var_map: &GuardedVarMap, amb: &mut Ambient<'_>) -> ExprRet {
    match expr {
        Expr::Eq(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (ExprRet::Value(left), ExprRet::Value(right)) => {
                    match (left.borrow(), right.borrow()) {
                        (Bool(left), Bool(right)) => ExprRet::Value(Bool(left == right)),
                        (String(left), String(right)) => ExprRet::Value(Bool(left == right)),
                        _ => panic!("incompatible types")
                    }
                }
                (_, _) => panic!("unexpected value")
            }
        }
        Expr::Assign(var_name, value) => {
            let value = match eval_expr(value, var_map, amb).await {
                ExprRet::Value(v) => v,
                ExprRet::Void => panic!("unexpected value")
            };

            var_map.lock().unwrap().scope_values.insert(var_name.clone(), value);
            return ExprRet::Void;
        }
        Expr::KeyMapping(mappings) => {
            for mapping in mappings {
                let mut mapping = mapping.clone();
                mapping.to.var_map = var_map.clone();

                amb.message_tx.borrow_mut().as_ref().unwrap()
                    .send(ExecutionMessage::AddMapping(amb.window_cycle_token, mapping.from, mapping.to)).await
                    .unwrap();
            }

            return ExprRet::Void;
        }
        Expr::Name(var_name) => {
            let mut value = None;
            let mut map = var_map.clone();

            loop {
                let tmp;
                let map_guard = map.lock().unwrap();
                match map_guard.scope_values.get(var_name) {
                    Some(v) => {
                        value = Some(v.clone());
                        break;
                    }
                    None => match &map_guard.parent {
                        Some(parent) => tmp = parent.clone(),
                        None => { panic!(format!("variable '{}' not found", var_name)); }
                    }
                }
                drop(map_guard);
                map = tmp;
            }

            return ExprRet::Value(value.unwrap());
        }
        Expr::Boolean(value) => {
            return ExprRet::Value(ValueType::Bool(*value));
        }
        Expr::String(value) => {
            return ExprRet::Value(ValueType::String(value.clone()));
        }
        Expr::KeyAction(action) => {
            amb.ev_writer_tx.send(action.to_input_ev()).await;
            amb.ev_writer_tx.send(SYN_REPORT.clone()).await;

            return ExprRet::Void;
        }
        Expr::EatKeyAction(action) => {
            match &amb.message_tx {
                Some(tx) => { tx.send(ExecutionMessage::EatEv(action.clone())).await.unwrap(); }
                None => panic!("need message tx"),
            }
            return ExprRet::Void;
        }
        Expr::SleepAction(duration) => {
            tokio::time::sleep(*duration).await;
            return ExprRet::Void;
        }
        Expr::FunctionCall(name, args) => {
            match &**name {
                "active_window_class" => {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(0);
                    amb.message_tx.as_ref().unwrap().send(ExecutionMessage::GetFocusedWindowInfo(tx)).await.unwrap();

                    if let Some(active_window) = rx.recv().await.unwrap() {
                        return ExprRet::Value(ValueType::String(active_window.class));
                    }
                    ExprRet::Void
                }
                _ => ExprRet::Void
            }
        }
    }
}

pub(crate) type SleepSender = tokio::sync::mpsc::Sender<Block>;

pub(crate) struct Ambient<'a> {
    pub(crate) ev_writer_tx: mpsc::Sender<InputEvent>,
    pub(crate) message_tx: Option<&'a mut ExecutionMessageSender>,
    pub(crate) window_cycle_token: usize,
}

#[async_recursion]
pub(crate) async fn eval_block<'a>(block: &Block, amb: &mut Ambient<'a>) {
    // let var_map = block.var_map.clone();
    for stmt in &block.statements {
        log_msg(&format!("{:?}", stmt));
        match stmt {
            Stmt::Expr(expr) => { eval_expr(expr, &block.var_map, amb).await; }
            Stmt::Block(nested_block) => {
                eval_block(nested_block, amb).await;
            }
            Stmt::ConditionalBlock(condition, nested_block) => {
                // nested_block.var_map = Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: Some(block.var_map.clone()) }));
                eval_conditional_block(condition, nested_block, amb).await;
            }
            Stmt::If(expr, block) => {
                log_msg("wo");
                if eval_expr(expr, &block.var_map, amb).await == ExprRet::Value(ValueType::Bool(true)) {
                    log_msg("wo2");
                    eval_block(block, amb).await;
                }
            }
        }
    }
}

fn mutexes_are_equal<T>(first: &Mutex<T>, second: &Mutex<T>) -> bool
    where T: PartialEq { std::ptr::eq(first, second) || *first.lock().unwrap() == *second.lock().unwrap() }

fn arc_mutexes_are_equal<T>(first: &Arc<Mutex<T>>, second: &Arc<Mutex<T>>) -> bool
    where T: PartialEq { Arc::ptr_eq(first, second) || *first.lock().unwrap() == *second.lock().unwrap() }

#[derive(Clone, Debug)]
pub(crate) struct Block {
    pub(crate) var_map: GuardedVarMap,
    pub(crate) statements: Vec<Stmt>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements &&
            arc_mutexes_are_equal(&self.var_map, &other.var_map)
    }
}

impl Block {
    pub(crate) fn new() -> Self {
        Block {
            var_map: Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: None })),
            statements: vec![],
        }
    }

    pub(crate) fn attach_underlying_scope(&mut self, block: &mut Block) {
        block.var_map.lock().unwrap().parent = Some(self.var_map.clone());
    }

    pub(crate) fn push_block(&mut self, mut block: Block) {
        self.attach_underlying_scope(&mut block);
        self.statements.push(Stmt::Block(block));
    }
}


#[derive(Eq, PartialEq)]
pub(crate) enum ExprRet {
    Void,
    Value(ValueType),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Eq(Box<Expr>, Box<Expr>),
    // LT(Expr, Expr),
    // GT(Expr, Expr),
    // INC(Expr),
    // Add(Expr, Expr),
    Assign(String, Box<Expr>),
    KeyMapping(Vec<KeyMapping>),

    Name(String),
    Boolean(bool),
    String(String),

    FunctionCall(String, Vec<ValueType>),

    KeyAction(KeyAction),
    EatKeyAction(KeyAction),
    SleepAction(time::Duration),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expr(Expr),
    Block(Block),
    ConditionalBlock(KeyActionCondition, Block),
    If(Expr, Block),
    // While
    // For(Expr::Assign, Expr, Expr, Stmt::Block)
    // Return(Expr),
}