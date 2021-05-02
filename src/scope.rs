use std::borrow::{BorrowMut};
use std::fmt;
use std::fmt::Formatter;

use crate::*;
use crate::parsing::parser::parse_key_sequence;
use x11rb::protocol::xproto::lookup_color;
use evdev_rs::enums::int_to_ev_key;
use std::convert::{TryInto, TryFrom};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyActionCondition {
    pub(crate) window_class_name: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) enum ValueType {
    Bool(bool),
    String(String),
    Lambda(Block, GuardedVarMap),
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
            ValueType::Lambda(_, _) => write!(f, "Lambda"),
            ValueType::Void => write!(f, "Void"),
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
pub(crate) async fn eval_expr<'a>(expr: &Expr, var_map: &GuardedVarMap, amb: &mut Ambient<'_>) -> ValueType {
    match expr {
        Expr::Eq(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left == right),
                (String(left), String(right)) => Bool(left == right),
                (Number(left), Number(right)) => Bool(left == right),
                _ => Bool(false),
            }
        }
        Expr::Neq(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left != right),
                (String(left), String(right)) => Bool(left != right),
                (Number(left), Number(right)) => Bool(left != right),
                _ => Bool(true),
            }
        }
        Expr::LT(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left < right),
                (String(left), String(right)) => Bool(left < right),
                (Number(left), Number(right)) => Bool(left < right),
                _ => Bool(false),
            }
        }
        Expr::GT(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Bool(left), Bool(right)) => Bool(left > right),
                (String(left), String(right)) => Bool(left > right),
                (Number(left), Number(right)) => Bool(left > right),
                _ => Bool(false),
            }
        }
        Expr::Add(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Number(left), Number(right)) => Number(left + right),
                (String(left), String(right)) => String(format!("{}{}", left, right)),
                _ => panic!("cannot add unsupported types"),
            }
        }
        Expr::Sub(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, amb).await, eval_expr(right, var_map, amb).await) {
                (Number(left), Number(right)) => Number(left - right),
                _ => panic!("cannot subtract unsupported types"),
            }
        }
        Expr::Init(var_name, value) => {
            let value = match eval_expr(value, var_map, amb).await {
                ValueType::Void => panic!("unexpected value"),
                v => v,
            };

            var_map.lock().unwrap().scope_values.insert(var_name.clone(), value);
            return ValueType::Void;
        }
        Expr::Assign(var_name, value) => {
            let value = match eval_expr(value, var_map, amb).await {
                ValueType::Void => panic!("unexpected value"),
                v => v,
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
        Expr::Lambda(block) => {
            return ValueType::Lambda(block.clone(), var_map.clone());
        }
        Expr::KeyAction(action) => {
            amb.ev_writer_tx.send(action.to_input_ev()).await.unwrap();
            amb.ev_writer_tx.send(SYN_REPORT.clone()).await.unwrap();

            return ValueType::Void;
        }
        Expr::EatKeyAction(action) => {
            match &amb.message_tx {
                Some(tx) => { tx.send(ExecutionMessage::EatEv(action.clone())).await.unwrap(); }
                None => panic!("need message tx"),
            }
            return ValueType::Void;
        }
        Expr::SleepAction(duration) => {
            tokio::time::sleep(*duration).await;
            return ValueType::Void;
        }
        Expr::FunctionCall(name, args) => {
            match &**name {
                "send" => {
                    let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
                    let val = match val {
                        ValueType::String(val) => val,
                        _ => panic!("invalid parameter passed to function 'send'"),
                    };

                    let actions = parse_key_sequence(&*val).unwrap();

                    for action in actions {
                        amb.ev_writer_tx.send(action.to_input_ev()).await.unwrap();
                        amb.ev_writer_tx.send(SYN_REPORT.clone()).await.unwrap();
                    }

                    ValueType::Void
                }

                "active_window_class" => {
                    let (tx, mut rx) = mpsc::channel(1);
                    amb.message_tx.as_ref().unwrap().send(ExecutionMessage::GetFocusedWindowInfo(tx)).await.unwrap();
                    if let Some(active_window) = rx.recv().await.unwrap() {
                        return ValueType::String(active_window.class);
                    }
                    ValueType::Void
                }
                "on_window_change" => {
                    if args.len() != 1 { panic!("function takes 1 argument") }

                    let inner_block;
                    let inner_var_map;
                    if let ValueType::Lambda(_block, _var_map) = eval_expr(args.get(0).unwrap(), var_map, amb).await {
                        inner_block = _block;
                        inner_var_map = _var_map;
                    } else {
                        panic!("type mismatch, function takes lambda argument");
                    }

                    amb.message_tx.as_ref().unwrap().send(ExecutionMessage::RegisterWindowChangeCallback(inner_block, inner_var_map)).await.unwrap();
                    ValueType::Void
                }
                "print" => {
                    let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
                    println!("{}", val);
                    ValueType::Void
                }
                "number_to_key" => {
                    let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
                    let val = match val {
                        ValueType::Number(val) => val,
                        _ => panic!("only numbers can be converted to keys"),
                    };
                    let val = val as u32;

                    let key = int_to_ev_key(val).expect(&*format!("key for scan code '{}' not found", val));

                    ValueType::String(format!("{{{}}}", EventCode::EV_KEY(key).to_string()))
                }
                "number_to_char" => {
                    let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
                    let val = match val {
                        ValueType::Number(val) => val,
                        _ => panic!("only numbers can be converted to chars"),
                    };

                    let val = val as u8 as char;
                    ValueType::String(format!("{}", val))
                }
                _ => ValueType::Void
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

    'outer: for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => { eval_expr(expr, &mut var_map, amb).await; }
            Stmt::Block(nested_block) => {
                eval_block(nested_block, &mut var_map, amb).await;
            }
            Stmt::If(if_else_if_pairs, else_pair) => {
                for (expr, block) in if_else_if_pairs {
                    if eval_expr(expr, &mut var_map, amb).await == ValueType::Bool(true) {
                        eval_block(block, &mut var_map, amb).await;
                        continue 'outer;
                    }
                }
                if let Some(block) = else_pair {
                    eval_block(block, &mut var_map, amb).await;
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

                    eval_block(block, &mut var_map, amb).await;
                    eval_expr(advance_expr, &var_map, amb).await;
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


#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    LT(Box<Expr>, Box<Expr>),
    GT(Box<Expr>, Box<Expr>),
    // INC(Expr),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Init(String, Box<Expr>),
    Assign(String, Box<Expr>),
    KeyMapping(Vec<KeyMapping>),

    Name(String),
    Value(ValueType),
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
    If(Vec<(Expr, Block)>, Option<Block>),
    For(Expr, Expr, Expr, Block),
    // While
    // Return(Expr),
}