use std::borrow::BorrowMut;
use std::fmt;
use std::fmt::Formatter;

use messaging::*;

use crate::*;

use super::builtin_functions::evaluate_builtin;
use super::builtin_functions::throw_error;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyActionCondition {
    pub(crate) window_class_name: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ValueType {
    Bool(bool),
    String(String),
    Lambda(Vec<String>, Block, GuardedVarMap),
    Number(f64),
    Void,
}

impl PartialEq for ValueType {
    fn eq(&self, other: &Self) -> bool {
        use ValueType::*;
        match (self, other) {
            (String(l), String(r)) => l == r,
            (Bool(l), Bool(r)) => l == r,
            (Number(l), Number(r)) => l == r,
            (_, _) => false,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Bool(v) => write!(f, "{}", v),
            ValueType::String(v) => write!(f, "{}", v),
            ValueType::Number(v) => write!(f, "{}", v),
            ValueType::Lambda(_, _, _) => write!(f, "Lambda"),
            ValueType::Void => write!(f, "Void"),
        }
    }
}

#[derive(Debug)]
pub struct VarMap {
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

pub type GuardedVarMap = Arc<Mutex<VarMap>>;


#[async_recursion]
pub(crate) async fn eval_expr<'a>(expr: &Expr, var_map: &GuardedVarMap, amb: &mut Ambient<'_>) -> ValueType {
    use ValueType::*;
    match expr {
        Expr::Eq(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left == right),
                (String(left), String(right)) => Bool(left == right),
                (Number(left), Number(right)) => Bool(left == right),
                _ => Bool(false),
            }
        }
        Expr::Neq(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left != right),
                (String(left), String(right)) => Bool(left != right),
                (Number(left), Number(right)) => Bool(left != right),
                _ => Bool(true),
            }
        }
        Expr::LT(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left < right),
                (String(left), String(right)) => Bool(left < right),
                (Number(left), Number(right)) => Bool(left < right),
                _ => Bool(false),
            }
        }
        Expr::GT(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left > right),
                (String(left), String(right)) => Bool(left > right),
                (Number(left), Number(right)) => Bool(left > right),
                _ => Bool(false),
            }
        }
        Expr::Add(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Number(left), Number(right)) => Number(left + right),
                (String(left), right) => String(format!("{}{}", left, right)),
                (left, String(right)) => String(format!("{}{}", left, right)),
                _ => panic!("cannot add unsupported types"),
            }
        }
        Expr::Sub(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Number(left), Number(right)) => Number(left - right),
                _ => panic!("cannot subtract unsupported types"),
            }
        }
        Expr::Mul(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Number(left), Number(right)) => Number(left * right),
                _ => panic!("cannot multiply unsupported types"),
            }
        }
        Expr::Div(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Number(left), Number(right)) => {
                    if right == 0.0 { panic!("error: division by zero"); }
                    Number(left / right)
                }
                _ => panic!("cannot multiply unsupported types"),
            }
        }
        Expr::Neg(expr) => {
            match eval_expr(expr, var_map, amb).await {
                Bool(val) => { Bool(!val) }
                _ => panic!("cannot negate unsupported type"),
            }
        }
        Expr::And(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left == right),
                _ => panic!("cannot perform \"and\" operation on unsupported types"),
            }
        }
        Expr::Or(left, right) => {
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left || right),
                _ => panic!("cannot perform \"or\" operation on unsupported types"),
            }
        }
        Expr::Init(var_name, value) => {
            let value = eval_expr(value, var_map, amb).await;

            var_map.lock().unwrap().scope_values.insert(var_name.clone(), value);
            return ValueType::Void;
        }
        Expr::Assign(var_name, value) => {
            let value = eval_expr(value, var_map, amb).await;

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
            ValueType::Void
        }
        Expr::KeyMapping(mappings) => {
            for mapping in mappings {
                let mapping = mapping.clone();

                amb.message_tx.borrow_mut().as_ref().unwrap()
                    .send(ExecutionMessage::AddMapping(amb.window_cycle_token, mapping.from, mapping.to, var_map.clone())).await
                    .unwrap();
            }

            return ValueType::Void;
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
                Some(value) => value,
                None => ValueType::Void,
            }
        }
        Expr::Value(value) => {
            return value.clone();
        }
        Expr::Lambda(params, block) => {
            let lambda_var_map = GuardedVarMap::new(Mutex::new(VarMap::new(Some(var_map.clone()))));
            return ValueType::Lambda(params.clone(), block.clone(), lambda_var_map);
        }
        Expr::KeyAction(action) => {
            amb.ev_writer_tx.send(action.to_input_ev()).await.unwrap();
            amb.ev_writer_tx.send(SYN_REPORT.clone()).await.unwrap();

            return ValueType::Void;
        }
        // Expr::EatKeyAction(action) => {
        //     match &amb.message_tx {
        //         Some(tx) => { tx.send(ExecutionMessage::EatEv(action.clone())).await.unwrap(); }
        //         None => panic!("need message tx"),
        //     }
        //     return ValueType::Void;
        // }
        Expr::SleepAction(duration) => {
            tokio::time::sleep(*duration).await;
            return ValueType::Void;
        }
        Expr::FunctionCall(name, args) => {
            match evaluate_builtin(name, args, var_map, amb).await {
                Ok(v) => v,
                Err(err) => {
                    throw_error(err, 1, amb).await;
                    ValueType::Void
                }
            }
        }
        Expr::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
            let actual_state = &amb.modifier_state;

            // takes into account the actual state of a modifier and decides whether to release/restore it or not
            let release_or_restore_modifier = |is_actual_down: &bool, key: &Key| {
                if *to_type == 1 { // restore mods if actual mod is still pressed
                    if *is_actual_down {
                        futures::executor::block_on(amb.ev_writer_tx.send(
                            KeyAction { key: *key, value: *to_type }.to_input_ev()
                        )).unwrap();
                    }
                } else { // release mods if actual mod is still pressed (prob. always true since it was necessary to trigger the mapping)
                    if *is_actual_down {
                        futures::executor::block_on(amb.ev_writer_tx.send(
                            KeyAction { key: *key, value: *to_type }.to_input_ev()
                        )).unwrap();
                    }
                }
            };

            if from_flags.ctrl && !to_flags.ctrl {
                release_or_restore_modifier(&actual_state.left_ctrl, &*KEY_LEFT_CTRL);
                release_or_restore_modifier(&actual_state.right_ctrl, &*KEY_RIGHT_CTRL);
            }
            if from_flags.shift && !to_flags.shift {
                release_or_restore_modifier(&actual_state.left_shift, &*KEY_LEFT_SHIFT);
                release_or_restore_modifier(&actual_state.right_shift, &*KEY_RIGHT_SHIFT);
            }
            if from_flags.alt && !to_flags.alt {
                release_or_restore_modifier(&actual_state.left_alt, &*KEY_LEFT_ALT);
                release_or_restore_modifier(&actual_state.right_alt, &*KEY_RIGHT_ALT);
            }
            if from_flags.meta && !to_flags.meta {
                release_or_restore_modifier(&actual_state.left_meta, &*KEY_LEFT_META);
                release_or_restore_modifier(&actual_state.right_meta, &*KEY_RIGHT_META);
            }

            // TODO eat keys we just released, un-eat keys we just restored

            return ValueType::Void;
        }
    }
}

pub type SleepSender = tokio::sync::mpsc::Sender<Block>;

pub struct Ambient<'a> {
    pub ev_writer_tx: mpsc::Sender<InputEvent>,
    pub message_tx: Option<&'a mut ExecutionMessageSender>,
    pub window_cycle_token: usize,
    pub modifier_state: &'a KeyModifierState,
}

pub enum BlockRet {
    None,
    Continue,
    Return(ValueType),
}

#[async_recursion]
pub async fn eval_block<'a>(block: &Block, var_map: &GuardedVarMap, amb: &mut Ambient<'a>) -> BlockRet {
    let mut var_map = GuardedVarMap::new(Mutex::new(VarMap::new(Some(var_map.clone()))));

    'outer: for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => { eval_expr(expr, &var_map, amb).await; }
            Stmt::Block(nested_block) => {
                let ret = eval_block(nested_block, &mut var_map, amb).await;
                match ret {
                    BlockRet::None => {}
                    _ => return ret,
                };
            }
            Stmt::If(if_else_if_pairs, else_pair) => {
                for (expr, block) in if_else_if_pairs {
                    if eval_expr(expr, &mut var_map, amb).await == ValueType::Bool(true) {
                        let ret = eval_block(block, &mut var_map, amb).await;
                        match ret {
                            BlockRet::None => {}
                            _ => return ret,
                        };
                        continue 'outer;
                    }
                }
                if let Some(block) = else_pair {
                    let ret = eval_block(block, &mut var_map, amb).await;
                    match ret {
                        BlockRet::None => {}
                        _ => return ret,
                    };
                }
            }
            Stmt::For(init_expr, termination_expr, advance_expr, block) => {
                eval_expr(init_expr, &var_map, amb).await;

                loop {
                    let should_continue = match eval_expr(termination_expr, &var_map, amb).await {
                        ValueType::Bool(v) => v,
                        _ => panic!("termination condition in for loop needs to return a boolean"),
                    };
                    if !should_continue { break; }

                    let ret = eval_block(block, &mut var_map, amb).await;
                    match ret {
                        BlockRet::Return(_) => return ret,
                        _ => {}
                    };

                    eval_expr(advance_expr, &var_map, amb).await;
                }
            }
            Stmt::Return(expr) => {
                return BlockRet::Return(eval_expr(expr, &var_map, amb).await);
            }
            Stmt::Continue => {
                return BlockRet::Continue;
            }
        }
    }

    BlockRet::None
}

fn arc_mutexes_are_equal<T>(first: &Arc<Mutex<T>>, second: &Arc<Mutex<T>>) -> bool
    where T: PartialEq { Arc::ptr_eq(first, second) || *first.lock().unwrap() == *second.lock().unwrap() }

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub(crate) statements: Vec<Stmt>,
}

impl Block {
    pub(crate) fn new() -> Self {
        Block { statements: vec![] }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    LT(Box<Expr>, Box<Expr>),
    GT(Box<Expr>, Box<Expr>),
    // Inc(Expr),
    // Dec(Expr),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Init(String, Box<Expr>),
    Assign(String, Box<Expr>),
    KeyMapping(Vec<KeyMapping>),

    Name(String),
    Value(ValueType),
    Lambda(Vec<String>, Block),

    FunctionCall(String, Vec<Expr>),

    KeyAction(KeyAction),
    SleepAction(time::Duration),

    // EatKeyAction(KeyAction),

    // internal
    ReleaseRestoreModifiers(KeyModifierFlags, KeyModifierFlags, i32),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expr(Expr),
    Block(Block),
    If(Vec<(Expr, Block)>, Option<Block>),
    For(Expr, Expr, Expr, Block),
    // While
    Return(Expr),
    Continue,
}