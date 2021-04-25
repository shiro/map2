use std::borrow::{Borrow, BorrowMut};
use std::fmt;

use crate::*;
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyActionCondition { pub(crate) window_class_name: Option<String> }

#[derive(Clone, Debug)]
pub(crate) enum ValueType {
    Bool(bool),
    String(String),
    Lambda(Block, GuardedVarMap),
}

impl PartialEq for ValueType {
    fn eq(&self, other: &Self) -> bool {
        use ValueType::*;
        match (self, other) {
            (String(l), String(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (_, _) => false,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Bool(v) => write!(f, "{}", v),
            ValueType::String(v) => write!(f, "{}", v),
            ValueType::Lambda(_, _) => write!(f, "Lambda"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct VarMap {
    pub(crate) scope_values: HashMap<String, ValueType>,
    pub(crate) parent: Option<GuardedVarMap>,
}

impl VarMap {
    pub fn new(parent: Option<GuardedVarMap>) -> Self {
        VarMap { scope_values: Default::default(), parent }
    }
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
pub(crate) async fn eval_expr<'a>(expr: &Expr, var_map: &GuardedVarMap, amb: &mut Ambient<'_>) -> ExprRet {
    match expr {
        Expr::Eq(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (ExprRet::Value(left), ExprRet::Value(right)) => {
                    match (left.borrow(), right.borrow()) {
                        (Bool(left), Bool(right)) => ExprRet::Value(Bool(left == right)),
                        (String(left), String(right)) => ExprRet::Value(Bool(left == right)),
                        _ => ExprRet::Value(Bool(false)),
                    }
                }
                (_, _) => ExprRet::Value(Bool(false)),
            }
        }
        Expr::Init(var_name, value) => {
            let value = match eval_expr(value, var_map, amb).await {
                ExprRet::Value(v) => v,
                ExprRet::Void => panic!("unexpected value")
            };

            var_map.lock().unwrap().scope_values.insert(var_name.clone(), value);
            return ExprRet::Void;
        }
        Expr::Assign(var_name, value) => {
            let value = match eval_expr(value, var_map, amb).await {
                ExprRet::Value(v) => v,
                ExprRet::Void => panic!("unexpected value")
            };

            let mut map = var_map.clone();
            loop {
                let tmp;
                let mut map_guard = map.lock().unwrap();
                match map_guard.scope_values.get_mut(var_name) {
                    Some(v) => {
                        *v = value;
                        break;
                    }
                    None => match &map_guard.parent {
                        Some(parent) => tmp = parent.clone(),
                        None => { panic!("variable '{}' does not exist", var_name); }
                    }
                }
                drop(map_guard);
                map = tmp;
            }
            ExprRet::Void
        }
        Expr::KeyMapping(mappings) => {
            for mapping in mappings {
                let mut mapping = mapping.clone();

                amb.message_tx.borrow_mut().as_ref().unwrap()
                    .send(ExecutionMessage::AddMapping(amb.window_cycle_token, mapping.from, mapping.to, var_map.clone())).await
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
                        None => { break; }
                    }
                }
                drop(map_guard);
                map = tmp;
            }

            match value {
                Some(value) => ExprRet::Value(value),
                None => ExprRet::Void,
            }
        }
        Expr::Boolean(value) => {
            return ExprRet::Value(ValueType::Bool(*value));
        }
        Expr::String(value) => {
            return ExprRet::Value(ValueType::String(value.clone()));
        }
        Expr::Lambda(block) => {
            return ExprRet::Value(ValueType::Lambda(block.clone(), var_map.clone()));
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
                    let (tx, mut rx) = mpsc::channel(1);
                    amb.message_tx.as_ref().unwrap().send(ExecutionMessage::GetFocusedWindowInfo(tx)).await.unwrap();
                    if let Some(active_window) = rx.recv().await.unwrap() {
                        return ExprRet::Value(ValueType::String(active_window.class));
                    }
                    ExprRet::Void
                }
                "on_window_change" => {
                    if args.len() != 1 { panic!("function takes 1 argument") }

                    let inner_block;
                    let inner_var_map;
                    if let ExprRet::Value(ValueType::Lambda(_block, _var_map)) = eval_expr(args.get(0).unwrap(), var_map, amb).await {
                        inner_block = _block;
                        inner_var_map = _var_map;
                    } else {
                        panic!("type mismatch, function takes lambda argument");
                    }

                    amb.message_tx.as_ref().unwrap().send(ExecutionMessage::RegisterWindowChangeCallback(inner_block, inner_var_map)).await.unwrap();
                    ExprRet::Void
                }
                "print" => {
                    let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
                    println!("{}", val);
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
pub(crate) async fn eval_block<'a>(block: &Block, var_map: &mut GuardedVarMap, amb: &mut Ambient<'a>) {
    let mut var_map = GuardedVarMap::new(Mutex::new(VarMap::new(Some(var_map.clone()))));

    for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => { eval_expr(expr, &mut var_map, amb).await; }
            Stmt::Block(nested_block) => {
                eval_block(nested_block, &mut var_map, amb).await;
            }
            Stmt::If(expr, block) => {
                if eval_expr(expr, &mut var_map, amb).await == ExprRet::Value(ValueType::Bool(true)) {
                    eval_block(block, &mut var_map, amb).await;
                }
            }
        }
    }
}

fn mutexes_are_equal<T>(first: &Mutex<T>, second: &Mutex<T>) -> bool
    where T: PartialEq { std::ptr::eq(first, second) || *first.lock().unwrap() == *second.lock().unwrap() }

fn arc_mutexes_are_equal<T>(first: &Arc<Mutex<T>>, second: &Arc<Mutex<T>>) -> bool
    where T: PartialEq { Arc::ptr_eq(first, second) || *first.lock().unwrap() == *second.lock().unwrap() }

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Block {
    // pub(crate) var_map: GuardedVarMap,
    pub(crate) statements: Vec<Stmt>,
}

impl Block {
    pub(crate) fn new() -> Self {
        Block {
            // var_map: Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: None })),
            statements: vec![],
        }
    }
}


#[derive(PartialEq)]
pub(crate) enum ExprRet {
    Void,
    Value(ValueType),
}

impl fmt::Display for ExprRet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExprRet::Void => write!(f, "Void"),
            ExprRet::Value(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Eq(Box<Expr>, Box<Expr>),
    // LT(Expr, Expr),
    // GT(Expr, Expr),
    // INC(Expr),
    // Add(Expr, Expr),
    Init(String, Box<Expr>),
    Assign(String, Box<Expr>),
    KeyMapping(Vec<KeyMapping>),

    Name(String),
    Boolean(bool),
    String(String),
    Lambda(Block),

    FunctionCall(String, Vec<Expr>),

    KeyAction(KeyAction),
    EatKeyAction(KeyAction),
    SleepAction(time::Duration),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expr(Expr),
    Block(Block),
    If(Expr, Block),
    // While
    // For(Expr::Assign, Expr, Expr, Stmt::Block)
    // Return(Expr),
}