use crate::*;
use std::borrow::Borrow;


#[derive(Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyActionCondition { pub(crate) window_class_name: Option<String> }

#[derive(Clone, Debug, Hash)]
pub(crate) enum ValueType {
    Bool(bool),
}

#[derive(Debug)]
pub(crate) struct VarMap {
    pub(crate) scope_values: HashMap<String, ValueType>,
    pub(crate) parent: Option<GuardedVarMap>,
}

// pub(crate) enum ScopeInstruction {
//     Scope(Scope),
//     KeyMapping(KeyMappings),
//     Assignment(String, ValueType),
// }

pub(crate) type GuardedVarMap = Arc<Mutex<VarMap>>;

// pub struct Scope {
//     pub(crate) var_map: GuardedVarMap,
//     pub(crate) condition: Option<KeyActionCondition>,
//     pub(crate) instructions: Block,
// }

// impl Scope {
//     pub(crate) fn new() -> Self {
//         Scope {
//             var_map: Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: None })),
//             condition: None,
//             instructions: vec![],
//         }
//     }
//     pub(crate) fn push_scope(&mut self, mut scope: Scope) {
//         scope.var_map.lock().unwrap().parent = Some(self.var_map.clone());
//         self.instructions.push(ScopeInstruction::Scope(scope));
//     }
// }
// pub(crate) type ScopeCache = HashMap<String, KeyMappings>;


pub(crate) fn eval_conditional_block(condition: &KeyActionCondition, block: &Block, state: &mut State, amb: &mut Ambient) {
    // check condition
    if let Some(window_class_name) = &condition.window_class_name {
        if let Some(active_window) = &state.active_window {
            if *window_class_name != active_window.class {
                return;
            }
        } else {
            return;
        }
    }

    eval_block(block, state, amb);
}

pub(crate) fn eval_expr(expr: &Expr, var_map: &GuardedVarMap, state: &mut State, amb: &mut Ambient) -> ExprRet {
    match expr {
        Expr::Eq(left, right) => {
            use ValueType::*;
            match (eval_expr(left, var_map, state, amb), eval_expr(right, var_map, state, amb)) {
                (ExprRet::Value(left), ExprRet::Value(right)) => {
                    match (left.borrow(), right.borrow()) {
                        (Bool(left), Bool(right)) => ExprRet::Value(Bool(left == right)),
                        _ => panic!("incompatible types")
                    }
                }
                (_, _) => panic!("unexpected value")
            }
        }
        Expr::Assign(var_name, value) => {
            let value = match eval_expr(value, var_map, state, amb) {
                ExprRet::Value(v) => v,
                ExprRet::Void => panic!("unexpected value")
            };

            var_map.lock().unwrap().scope_values.insert(var_name.clone(), value);
            return ExprRet::Void;
        }
        Expr::KeyMapping(mapping) => {
            if let Some(mappings) = amb.mappings {
                let mut block = mapping.to.clone();
                block.var_map = var_map.clone();
                mappings.0.insert(mapping.from, block);
            }
            return ExprRet::Void;
        }
        Expr::Name(var_name) => {
            let value = var_map.lock().unwrap().scope_values.get(var_name).unwrap().clone();
            return ExprRet::Value(value);
        }
        Expr::Boolean(value) => {
            return ExprRet::Value(ValueType::Bool(*value));
        }
        Expr::KeyAction(action) => {
            print_event(&action.to_input_ev());
            print_event(&INPUT_EV_SYN);
            thread::sleep(time::Duration::from_micros(20000));

            return ExprRet::Void;
        }
        Expr::EatKeyAction(action) => {
            state.ignore_list.ignore(&action);
            return ExprRet::Void;
        }
        Expr::SleepAction(duration) => {
            let duration = duration.clone();
            // let seq = KeySequence(to_action_seq.0.iter().skip(idx + 1).map(|v| v.clone()).collect());
            // let _var_map = var_map.clone();

            // let mut new_block = Block::new();
            // new_block.var_map = var_map.clone();
            // new_block.statements = block.statements

            // let delay_tx = amb.sleep_tx.unwrap().clone();
            tokio::spawn(async move {
                tokio::time::sleep(duration).await;
                // delay_tx.send((seq, _var_map)).await;
                Ok::<(), Error>(())
            });
            return ExprRet::Void;
        }
    }
}

pub(crate) type SleepSender = tokio::sync::mpsc::Sender<Block>;

pub(crate) struct Ambient<'a> {
    pub(crate) mappings: &'a mut Option<&'a mut CompiledKeyMappings>,
    pub(crate) sleep_tx: Option<&'a mut SleepSender>,
}

pub(crate) fn eval_block(block: &Block, state: &mut State, amb: &mut Ambient) {
    // let var_map = block.var_map.clone();
    for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => { eval_expr(expr, &block.var_map, state, amb); }
            Stmt::Block(nested_block) => {
                // TODO do this on AST creation
                // nested_block.var_map = Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: Some(block.var_map.clone()) }));
                eval_block(nested_block, state, amb);
            }
            Stmt::ConditionalBlock(condition, nested_block) => {
                // nested_block.var_map = Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: Some(block.var_map.clone()) }));
                eval_conditional_block(condition, nested_block, state, amb);
            }
        }
    }
}

// pub(crate) struct ConditionEq<T: Eq> {
//     left: T,
//     right: T,
// }


#[derive(Clone)]
pub(crate) struct Block {
    pub(crate) var_map: GuardedVarMap,
    pub(crate) statements: Vec<Stmt>,
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


pub(crate) enum ExprRet {
    Void,
    Value(ValueType),
}

#[derive(Clone)]
pub(crate) enum Expr {
    Eq(Box<Expr>, Box<Expr>),
    // LT(Expr, Expr),
    // GT(Expr, Expr),
    // INC(Expr),
    // Add(Expr, Expr),
    Assign(String, Box<Expr>),
    KeyMapping(KeyMapping),

    Name(String),
    Boolean(bool),

    KeyAction(KeyAction),
    EatKeyAction(KeyAction),
    SleepAction(time::Duration),
}

#[derive(Clone)]
pub(crate) enum Stmt {
    Expr(Expr),
    Block(Block),
    ConditionalBlock(KeyActionCondition, Block),
    // While
    // For(Expr::Assign, Expr, Expr, Stmt::Block)
    // Return(Expr),
}