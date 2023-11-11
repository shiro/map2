use crate::*;


#[derive(Debug)]
pub enum InputEvent {
    Raw(EvdevInputEvent)
}