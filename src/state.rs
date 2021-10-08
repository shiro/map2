use crate::*;

pub struct State {
    pub modifiers: Arc<KeyModifierState>,
    pub active_window: Option<ActiveWindowInfo>,
}


impl State {
    pub fn new() -> Self {
        State {
            modifiers: Arc::new(KeyModifierState::new()),
            active_window: None,
        }
    }
}