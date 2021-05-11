use ignore_list::*;

use crate::*;

#[derive(Clone)]
pub struct CompiledKeyMappings(pub HashMap<KeyActionWithMods, Arc<tokio::sync::Mutex<(Block, GuardedVarMap)>>>);

impl CompiledKeyMappings { pub fn new() -> Self { CompiledKeyMappings(Default::default()) } }

pub struct State {
    pub tab_is_down: bool,
    pub capslock_is_down: bool,
    pub leftcontrol_is_down: bool,
    pub shift_is_down: bool,
    pub meta_is_down: bool,
    pub leftalt_is_down: bool,
    pub right_alt_is_down: bool,

    pub ignore_list: IgnoreList,
    pub active_window: Option<ActiveWindowInfo>,
}


impl State {
    pub fn new() -> Self {
        State {
            tab_is_down: false,
            capslock_is_down: false,
            leftcontrol_is_down: false,
            shift_is_down: false,
            meta_is_down: false,
            leftalt_is_down: false,
            right_alt_is_down: false,
            ignore_list: IgnoreList::new(),
            active_window: None,
        }
    }
}