use crate::*;

#[derive(Debug, Clone)]
pub enum InputEvent {
    Raw(EvdevInputEvent),
}
