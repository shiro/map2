use crate::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Key { pub(crate) key_type: i32, pub(crate) code: i32 }

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum KeyModifierStateType { KEEP, UP, DOWN }

impl KeyModifierStateType {
    fn to_event_value(&self) -> i32 {
        match self {
            KeyModifierStateType::KEEP => 2,
            KeyModifierStateType::UP => 0,
            KeyModifierStateType::DOWN => 1
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct KeyModifierFlags {
    pub(crate) ctrl: bool,
    pub(crate) shift: bool,
    pub(crate) alt: bool,
    pub(crate) meta: bool,
}

impl KeyModifierFlags {
    pub fn new() -> Self { KeyModifierFlags { ctrl: false, shift: false, alt: false, meta: false } }
    pub fn ctrl(&mut self) -> &mut Self {
        self.ctrl = true;
        self
    }
    pub fn alt(&mut self) -> &mut Self {
        self.alt = true;
        self
    }
    pub fn shift(&mut self) -> &mut Self {
        self.shift = true;
        self
    }
    pub fn meta(&mut self) -> &mut Self {
        self.meta = true;
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct KeyAction { pub(crate) key: Key, pub(crate) value: i32 }

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct KeyActionWithMods { pub(crate) key: Key, pub(crate) value: i32, pub(crate) modifiers: KeyModifierFlags }

impl KeyAction {
    pub fn new(key: Key, value: i32) -> Self { KeyAction { key, value } }
    pub fn to_input_ev(&self) -> input_event {
        input_event { type_: self.key.key_type as u16, code: self.key.code as u16, value: self.value, time: INPUT_EV_DUMMY_TIME }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct KeyClickAction { key: Key, modifiers: KeyModifierFlags }

impl KeyClickAction {
    pub fn new(key: Key) -> Self { KeyClickAction { key, modifiers: KeyModifierFlags::new() } }
    pub fn new_mods(key: Key, modifiers: KeyModifierFlags) -> Self { KeyClickAction { key, modifiers } }
}

#[derive(Clone)]
pub(crate) struct KeyMapping {
    pub(crate) from: KeyActionWithMods,
    pub(crate) to: Block,
}
