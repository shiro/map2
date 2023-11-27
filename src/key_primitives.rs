use evdev_rs::enums::{EventCode, EventType};

use crate::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Key {
    pub event_code: EventCode,
}

impl Key {
    pub fn from_str(ev_type: &EventType, s: &str) -> Result<Self> {
        match EventCode::from_str(ev_type, s) {
            Some(event_code) => { Ok(Key { event_code }) }
            None => { Err(anyhow!("key not found: '{}'", s)) }
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[allow(unused)]
pub enum KeyValue { KEEP, UP, DOWN }

impl KeyValue {
    #[allow(unused)]
    fn to_event_value(&self) -> i32 {
        match self {
            KeyValue::KEEP => 2,
            KeyValue::UP => 0,
            KeyValue::DOWN => 1
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct KeyModifierFlags {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub right_alt: bool,
    pub meta: bool,
}

impl KeyModifierFlags {
    pub fn new() -> Self { KeyModifierFlags { ctrl: false, shift: false, alt: false, right_alt: false, meta: false } }
    pub fn ctrl(&mut self) { self.ctrl = true; }
    pub fn alt(&mut self) { self.alt = true; }
    pub fn right_alt(&mut self) { self.right_alt = true; }
    pub fn shift(&mut self) { self.shift = true; }
    pub fn meta(&mut self) {
        self.meta = true;
    }
    pub fn apply_from(&mut self, other: &KeyModifierFlags) {
        if other.ctrl { self.ctrl(); }
        if other.alt { self.alt(); }
        if other.right_alt { self.right_alt(); }
        if other.shift { self.shift(); }
        if other.meta { self.meta(); }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct KeyModifierState {
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub left_shift: bool,
    pub right_shift: bool,
    pub left_alt: bool,
    pub right_alt: bool,
    pub left_meta: bool,
    pub right_meta: bool,
}

impl KeyModifierState {
    pub fn new() -> Self {
        KeyModifierState {
            left_ctrl: false,
            right_ctrl: false,
            left_shift: false,
            right_shift: false,
            left_alt: false,
            right_alt: false,
            left_meta: false,
            right_meta: false,
        }
    }
    pub fn is_ctrl(&self) -> bool { self.left_ctrl || self.right_ctrl }
    pub fn is_alt(&self) -> bool { self.left_alt }
    pub fn is_right_alt(&self) -> bool { self.right_alt }
    pub fn is_shift(&self) -> bool { self.left_shift || self.right_shift }
    pub fn is_meta(&self) -> bool { self.left_meta || self.right_meta }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyAction {
    pub key: Key,
    pub value: i32,
}

impl KeyAction {
    pub fn new(key: Key, value: i32) -> Self { KeyAction { key, value } }
    pub fn from_input_ev(ev: &EvdevInputEvent) -> Self { KeyAction { key: Key { event_code: ev.event_code }, value: ev.value } }
    pub fn to_input_ev(&self) -> EvdevInputEvent {
        EvdevInputEvent { event_code: self.key.event_code, value: self.value, time: INPUT_EV_DUMMY_TIME }
    }
    pub fn to_key_action_with_mods(self, modifiers: KeyModifierFlags) -> KeyActionWithMods {
        KeyActionWithMods { key: self.key, value: self.value, modifiers }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyActionWithMods {
    pub key: Key,
    pub value: i32,
    pub modifiers: KeyModifierFlags,
}

impl KeyActionWithMods {
    pub fn new(key: Key, value: i32, modifiers: KeyModifierFlags) -> Self { KeyActionWithMods { key, value, modifiers } }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyClickActionWithMods {
    pub key: Key,
    pub modifiers: KeyModifierFlags,
}

impl KeyClickActionWithMods {
    pub fn new(key: Key) -> Self { KeyClickActionWithMods { key, modifiers: KeyModifierFlags::new() } }
    pub fn new_with_mods(key: Key, modifiers: KeyModifierFlags) -> Self { KeyClickActionWithMods { key, modifiers } }
    pub fn to_key_action(self, value: i32) -> KeyActionWithMods { KeyActionWithMods::new(self.key, value, self.modifiers) }
}

pub static TYPE_UP: i32 = 0;
pub static TYPE_DOWN: i32 = 1;
pub static TYPE_REPEAT: i32 = 2;
