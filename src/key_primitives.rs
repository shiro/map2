use evdev_rs::enums::{EventCode, EventType};

use crate::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct Key {
    pub event_code: EventCode,
}

impl Key {
    pub fn from_str(key_name: &str) -> Result<Self> {
        let mut key_name = key_name.to_uppercase();

        // prefix if not already
        if !key_name.starts_with("KEY_") && !key_name.starts_with("BTN_") {
            key_name = "KEY_".to_string().tap_mut(|s| s.push_str(&key_name));
        }

        match EventCode::from_str(&EventType::EV_KEY, &key_name) {
            Some(event_code) => Ok(Key { event_code }),
            None => Err(anyhow!("key not found: '{}'", key_name)),
        }
    }

    pub fn to_input_ev(&self, state: i32) -> EvdevInputEvent {
        EvdevInputEvent::new(&Default::default(), &self.event_code, state)
    }
}

impl From<evdev_rs::enums::EV_KEY> for Key {
    fn from(value: evdev_rs::enums::EV_KEY) -> Self {
        Self { event_code: EventCode::EV_KEY(value) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[allow(unused)]
pub enum KeyValue {
    KEEP,
    UP,
    DOWN,
}

impl KeyValue {
    #[allow(unused)]
    fn to_event_value(&self) -> i32 {
        match self {
            KeyValue::KEEP => 2,
            KeyValue::UP => 0,
            KeyValue::DOWN => 1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct KeyModifierFlags {
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub left_shift: bool,
    pub right_shift: bool,
    pub left_alt: bool,
    pub right_alt: bool,
    pub left_meta: bool,
    pub right_meta: bool,
}

impl KeyModifierFlags {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn left_ctrl(&mut self) {
        self.left_ctrl = true;
    }
    pub fn right_ctrl(&mut self) {
        self.right_ctrl = true;
    }
    pub fn left_alt(&mut self) {
        self.left_alt = true;
    }
    pub fn right_alt(&mut self) {
        self.right_alt = true;
    }
    pub fn left_shift(&mut self) {
        self.left_shift = true;
    }
    pub fn right_shift(&mut self) {
        self.right_shift = true;
    }
    pub fn left_meta(&mut self) {
        self.left_meta = true;
    }
    pub fn right_meta(&mut self) {
        self.right_meta = true;
    }
    pub fn is_ctrl(&mut self) -> bool {
        self.left_ctrl || self.right_ctrl
    }
    pub fn is_shift(&mut self) -> bool {
        self.left_shift || self.right_shift
    }
    pub fn is_alt(&mut self) -> bool {
        self.left_alt || self.right_alt
    }
    pub fn is_meta(&mut self) -> bool {
        self.left_meta || self.right_meta
    }
    pub fn apply_from(&mut self, other: &KeyModifierFlags) {
        if other.left_ctrl {
            self.left_ctrl();
        }
        if other.right_ctrl {
            self.right_ctrl();
        }
        if other.left_alt {
            self.left_alt();
        }
        if other.right_alt {
            self.right_alt();
        }
        if other.left_shift {
            self.left_shift();
        }
        if other.right_shift {
            self.right_shift();
        }
        if other.left_meta {
            self.left_meta();
        }
        if other.right_meta {
            self.right_meta();
        }
    }
    pub fn update_from_action(&mut self, action: &KeyAction) {
        let value = action.value == 1;
        match action.key.event_code {
            EventCode::EV_KEY(key) => match key {
                KEY_LEFTCTRL => self.left_ctrl = value,
                KEY_RIGHTCTRL => self.right_ctrl = value,
                KEY_LEFTSHIFT => self.left_shift = value,
                KEY_RIGHTSHIFT => self.right_shift = value,
                KEY_LEFTALT => self.left_alt = value,
                KEY_RIGHTALT => self.right_alt = value,
                KEY_LEFTMETA => self.left_meta = value,
                KEY_RIGHTMETA => self.right_meta = value,
                _ => {}
            },
            _ => unreachable!(),
        };
    }
    pub fn hash(&self) -> u32 {
        let mut hash = 0;
        if self.left_ctrl || self.right_ctrl {
            hash = hash | (1 << 0);
        }
        if self.left_shift || self.right_shift {
            hash = hash | (1 << 1);
        }
        if self.left_alt || self.right_alt {
            hash = hash | (1 << 2);
        }
        if self.left_meta || self.right_meta {
            hash = hash | (1 << 3);
        }
        hash
    }
}

pub fn diff_modifiers_to_key_actions(from: &KeyModifierFlags, to: &KeyModifierFlags) -> Vec<KeyAction> {
    unimplemented!()
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
    pub fn is_ctrl(&self) -> bool {
        self.left_ctrl || self.right_ctrl
    }
    pub fn is_alt(&self) -> bool {
        self.left_alt
    }
    pub fn is_right_alt(&self) -> bool {
        self.right_alt
    }
    pub fn is_shift(&self) -> bool {
        self.left_shift || self.right_shift
    }
    pub fn is_meta(&self) -> bool {
        self.left_meta || self.right_meta
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyAction {
    pub key: Key,
    pub value: i32,
}

impl KeyAction {
    pub fn new(key: Key, value: i32) -> Self {
        KeyAction { key, value }
    }
    pub fn from_input_ev(ev: &EvdevInputEvent) -> Self {
        KeyAction { key: Key { event_code: ev.event_code }, value: ev.value }
    }
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
    pub fn new(key: Key, value: i32, modifiers: KeyModifierFlags) -> Self {
        KeyActionWithMods { key, value, modifiers }
    }
    pub fn to_key_action(self) -> KeyAction {
        KeyAction::new(self.key, self.value)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyClickActionWithMods {
    pub key: Key,
    pub modifiers: KeyModifierFlags,
}

impl KeyClickActionWithMods {
    pub fn new(key: Key) -> Self {
        KeyClickActionWithMods { key, modifiers: KeyModifierFlags::new() }
    }
    pub fn new_with_mods(key: Key, modifiers: KeyModifierFlags) -> Self {
        KeyClickActionWithMods { key, modifiers }
    }
    pub fn to_key_action_with_mods(self, value: i32) -> KeyActionWithMods {
        KeyActionWithMods::new(self.key, value, self.modifiers)
    }
}

pub const TYPE_UP: i32 = 0;
pub const TYPE_DOWN: i32 = 1;
pub const TYPE_REPEAT: i32 = 2;
