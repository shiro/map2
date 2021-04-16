use crate::*;

pub enum ScopeInstruction {
    Scope(Scope),
    KeyMapping(KeyMappings),
}


#[derive(Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyActionCondition { pub(crate) window_class_name: Option<String> }

pub struct Scope {
    pub(crate) condition: Option<KeyActionCondition>,
    pub(crate) instructions: Vec<ScopeInstruction>,
}

pub(crate) type ScopeCache = HashMap<String, KeyMappings>;

pub(crate) fn eval_scope(scope: &Scope, state: &mut State, cache: &mut ScopeCache) {
    if let Some(active_window) = &state.active_window {
        if let Some(cached) = cache.get(&active_window.class) {
            state.mappings.0.extend(
                cached.0.iter().map(|(k, v)| { (k.clone(), v.clone()) })
            );
            return;
        }
    }

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

    for instruction in &scope.instructions {
        match instruction {
            ScopeInstruction::Scope(sub_scope) => { eval_scope(sub_scope, state, cache); }
            ScopeInstruction::KeyMapping(mapping) => {
                state.mappings.0.extend(
                    mapping.0.iter().map(|(k, v)| { (k.clone(), v.clone()) })
                );
            }
        }
    }

    // cache for later
    if let Some(active_window) = &state.active_window {
        cache.insert(active_window.class.to_string(), state.mappings.clone());
    }
}
