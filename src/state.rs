use crate::*;

#[derive(Default)]
pub struct State {
    pub modifiers: Arc<KeyModifierState>,
}


impl State {
    pub fn new() -> Self {
        State {
            modifiers: Arc::new(KeyModifierState::new()),
        }
    }
}