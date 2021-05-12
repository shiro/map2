use ignore_list::*;

use crate::*;

#[derive(Clone)]
pub struct CompiledKeyMappings(pub HashMap<KeyActionWithMods, Arc<(Block, GuardedVarMap)>>);

impl CompiledKeyMappings { pub fn new() -> Self { CompiledKeyMappings(Default::default()) } }

pub struct State {
    pub modifiers: KeyModifierState,

    pub ignore_list: IgnoreList,
    pub active_window: Option<ActiveWindowInfo>,
}


impl State {
    pub fn new() -> Self {
        State {
            modifiers: KeyModifierState::new(),
            ignore_list: IgnoreList::new(),
            active_window: None,
        }
    }
}