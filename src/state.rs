use crate::*;

#[derive(Default)]
pub struct MapperState {
    pub modifiers: Arc<KeyModifierState>,
}

impl MapperState {
    pub fn new() -> Self {
        MapperState { modifiers: Arc::new(KeyModifierState::new()) }
    }
}
