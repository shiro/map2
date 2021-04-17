use crate::*;


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

pub(crate) enum ScopeInstruction {
    Scope(Scope),
    KeyMapping(KeyMappings),
    Assignment(String, ValueType),
}

pub(crate) type GuardedVarMap = Arc<Mutex<VarMap>>;

pub struct Scope {
    pub(crate) var_map: GuardedVarMap,
    pub(crate) condition: Option<KeyActionCondition>,
    pub(crate) instructions: Vec<ScopeInstruction>,
}

impl Scope {
    pub(crate) fn new() -> Self {
        Scope {
            var_map: Arc::new(Mutex::new(VarMap { scope_values: Default::default(), parent: None })),
            condition: None,
            instructions: vec![],
        }
    }
    pub(crate) fn push_scope(&mut self, mut scope: Scope) {
        scope.var_map.lock().unwrap().parent = Some(self.var_map.clone());
        self.instructions.push(ScopeInstruction::Scope(scope));
    }
}


pub(crate) type ScopeCache = HashMap<String, KeyMappings>;

pub(crate) fn eval_scope(scope: &mut Scope, state: &mut State, cache: &mut ScopeCache) {
    // if let Some(active_window) = &state.active_window {
    //     if let Some(cached) = cache.get(&active_window.class) {
    //         state.mappings.0.extend(
    //             cached.0.iter().map(|(k, v)| { (k.clone(), v.clone()) })
    //         );
    //         return;
    //     }
    // }

    // check condition
    if let Some(cond) = &scope.condition {
        if let Some(window_class_name) = &cond.window_class_name {
            if let Some(active_window) = &state.active_window {
                if *window_class_name != active_window.class {
                    return;
                }
            } else {
                return;
            }
        }
    }

    let var_map = scope.var_map.clone();
    for instruction in &mut scope.instructions {
        match instruction {
            ScopeInstruction::Scope(sub_scope) => { eval_scope(sub_scope, state, cache); }
            ScopeInstruction::KeyMapping(mapping) => {
                state.mappings.0.extend(
                    mapping.0.iter().map(|(k, v)| { (k.clone(), (v.clone(), var_map.clone())) })
                );
            }
            ScopeInstruction::Assignment(var_name, value) => {
                scope.var_map.lock().unwrap().scope_values.insert(var_name.clone(), value.clone());
            }
        }
    }

    // cache for later
    // if let Some(active_window) = &state.active_window {
    //     cache.insert(active_window.class.to_string(), state.mappings.clone());
    // }
}

// pub(crate) struct ConditionEq<T: Eq> {
//     left: T,
//     right: T,
// }


pub(crate) struct Block {
    pub(crate) var_map: GuardedVarMap,
    pub(crate) statements: Vec<Stmt>,
}

pub(crate) enum Expr {
    Eq(Expr, Expr),
    // LT(Expr, Expr),
    // GT(Expr, Expr),
    // INC(Expr),
    // Add(Expr, Expr),
    Assign(Expr::Name, Expr),
    KeyMapping(KeyMapping),

    Name(String),
    // variable name
    Boolean(bool),
}

pub(crate) enum Stmt {
    Expr(Expr),
    Block(Block),
    ConditionalBlock(Option<KeyActionCondition>, Block),

    // While
    // For(Expr::Assign, Expr, Expr, Stmt::Block)
    // Return(Expr),
}

pub(crate) enum Condition {
    Eq(String, String),
}