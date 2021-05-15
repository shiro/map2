use ignore_list::*;

use crate::*;

#[derive(Clone, Debug)]
pub struct CompiledKeyMappings(pub HashMap<KeyActionWithMods, Arc<(Block, GuardedVarMap)>>);

impl CompiledKeyMappings { pub fn new() -> Self { CompiledKeyMappings(Default::default()) } }

pub struct State {
    pub modifiers: Arc<KeyModifierState>,

    pub ignore_list: IgnoreList,
    pub active_window: Option<ActiveWindowInfo>,
}


impl State {
    pub fn new() -> Self {
        State {
            modifiers: Arc::new(KeyModifierState::new()),
            ignore_list: IgnoreList::new(),
            active_window: None,
        }
    }
}