use ignore_list::*;

use crate::*;

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