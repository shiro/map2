use crate::*;
use evdev_rs::enums::{EventCode, EventType};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Key { pub(crate) event_code: EventCode }

impl Key {
    pub(crate) fn from_str(ev_type: &EventType, s: &str) -> Result<Self> {
        match EventCode::from_str(ev_type, s) {
            Some(event_code) => { Ok(Key { event_code }) }
            None => { Err(anyhow!("key not found: '{}'", s)) }
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum KeyValue { KEEP, UP, DOWN }

impl KeyValue {
    fn to_event_value(&self) -> i32 {
        match self {
            KeyValue::KEEP => 2,
            KeyValue::UP => 0,
            KeyValue::DOWN => 1
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

impl KeyAction {
    pub fn new(key: Key, value: i32) -> Self { KeyAction { key, value } }
    pub fn to_input_ev(&self) -> InputEvent {
        InputEvent { event_code: self.key.event_code, value: self.value, time: INPUT_EV_DUMMY_TIME }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct KeyActionWithMods { pub(crate) key: Key, pub(crate) value: i32, pub(crate) modifiers: KeyModifierFlags }

impl KeyActionWithMods {
    pub fn new(key: Key, value: i32, modifiers: KeyModifierFlags) -> Self { KeyActionWithMods { key, value, modifiers } }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct KeyClickActionWithMods { pub(crate) key: Key, pub(crate) modifiers: KeyModifierFlags }

impl KeyClickActionWithMods {
    pub fn new(key: Key) -> Self { KeyClickActionWithMods { key, modifiers: KeyModifierFlags::new() } }
    pub fn new_with_mods(key: Key, modifiers: KeyModifierFlags) -> Self { KeyClickActionWithMods { key, modifiers } }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KeyMapping {
    pub(crate) from: KeyActionWithMods,
    pub(crate) to: Block,
}

pub(crate) static TYPE_UP: i32 = 0;
pub(crate) static TYPE_DOWN: i32 = 1;
pub(crate) static TYPE_REPEAT: i32 = 2;
