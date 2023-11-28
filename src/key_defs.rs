use evdev_rs::enums::{EV_SYN, EventCode};
use evdev_rs::TimeVal;

use crate::*;

pub const INPUT_EV_DUMMY_TIME: TimeVal = TimeVal { tv_sec: 0, tv_usec: 0 };

pub static SYN_REPORT: EvdevInputEvent = EvdevInputEvent { event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT), value: 0, time: INPUT_EV_DUMMY_TIME };

lazy_static! {
pub static ref BTN_LEFT: Key = Key::from_str("BTN_LEFT").unwrap();
pub static ref BTN_MIDDLE: Key = Key::from_str("BTN_MIDDLE").unwrap();
pub static ref BTN_RIGHT: Key = Key::from_str("BTN_RIGHT").unwrap();
pub static ref KEY_BACKSPACE: Key = Key::from_str("KEY_BACKSPACE").unwrap();
pub static ref KEY_DOWN: Key = Key::from_str("KEY_DOWN").unwrap();
pub static ref KEY_ESC: Key = Key::from_str("KEY_ESC").unwrap();
pub static ref KEY_KPD0: Key = Key::from_str("KEY_KP0").unwrap();
pub static ref KEY_KPD1: Key = Key::from_str("KEY_KP1").unwrap();
pub static ref KEY_KPD2: Key = Key::from_str("KEY_KP2").unwrap();
pub static ref KEY_KPD3: Key = Key::from_str("KEY_KP3").unwrap();
pub static ref KEY_KPD4: Key = Key::from_str("KEY_KP4").unwrap();
pub static ref KEY_KPD5: Key = Key::from_str("KEY_KP5").unwrap();
pub static ref KEY_KPD6: Key = Key::from_str("KEY_KP6").unwrap();
pub static ref KEY_KPD7: Key = Key::from_str("KEY_KP7").unwrap();
pub static ref KEY_KPD8: Key = Key::from_str("KEY_KP8").unwrap();
pub static ref KEY_KPD9: Key = Key::from_str("KEY_KP9").unwrap();
pub static ref KEY_LEFT: Key = Key::from_str("KEY_LEFT").unwrap();
pub static ref KEY_LEFTBRACE: Key = Key::from_str("KEY_LEFTBRACE").unwrap();
pub static ref KEY_LEFTSHIFT: Key = Key::from_str("KEY_LEFTSHIFT").unwrap();
pub static ref KEY_LEFTALT: Key = Key::from_str("KEY_LEFTALT").unwrap();
pub static ref KEY_LEFTCTRL: Key = Key::from_str("KEY_LEFTCTRL").unwrap();
pub static ref KEY_LEFTMETA: Key = Key::from_str("KEY_LEFTMETA").unwrap();
pub static ref KEY_PAGEDOWN: Key = Key::from_str("KEY_PAGEDOWN").unwrap();
pub static ref KEY_PAGEUP: Key = Key::from_str("KEY_PAGEUP").unwrap();
pub static ref KEY_RIGHT: Key = Key::from_str("KEY_RIGHT").unwrap();
pub static ref KEY_RIGHTBRACE: Key = Key::from_str("KEY_RIGHTBRACE").unwrap();
pub static ref KEY_RIGHTALT: Key = Key::from_str("KEY_RIGHTALT").unwrap();
pub static ref KEY_RIGHTCTRL: Key = Key::from_str("KEY_RIGHTCTRL").unwrap();
pub static ref KEY_RIGHTMETA: Key = Key::from_str("KEY_RIGHTMETA").unwrap();
pub static ref KEY_RIGHTSHIFT: Key = Key::from_str("KEY_RIGHTSHIFT").unwrap();
pub static ref KEY_UP: Key = Key::from_str("KEY_UP").unwrap();
}


lazy_static! {
    pub(crate) static ref KEY_ALIAS_TABLE: HashMap<&'static str, (Key, KeyModifierFlags)> = {
        let mut m = HashMap::new();
        m.insert("ALT", (*KEY_LEFTALT, KeyModifierFlags::new()));
        m.insert("CTRL", (*KEY_LEFTCTRL, KeyModifierFlags::new()));
        m.insert("ESCAPE", (*KEY_ESC, KeyModifierFlags::new()));
        m.insert("KEYPAD_0", (*KEY_KPD0, KeyModifierFlags::new()));
        m.insert("KEYPAD_1", (*KEY_KPD1, KeyModifierFlags::new()));
        m.insert("KEYPAD_2", (*KEY_KPD2, KeyModifierFlags::new()));
        m.insert("KEYPAD_3", (*KEY_KPD3, KeyModifierFlags::new()));
        m.insert("KEYPAD_4", (*KEY_KPD4, KeyModifierFlags::new()));
        m.insert("KEYPAD_5", (*KEY_KPD5, KeyModifierFlags::new()));
        m.insert("KEYPAD_6", (*KEY_KPD6, KeyModifierFlags::new()));
        m.insert("KEYPAD_7", (*KEY_KPD7, KeyModifierFlags::new()));
        m.insert("KEYPAD_8", (*KEY_KPD8, KeyModifierFlags::new()));
        m.insert("KEYPAD_9", (*KEY_KPD9, KeyModifierFlags::new()));
        m.insert("LEFT_ALT", (*KEY_LEFTALT, KeyModifierFlags::new()));
        m.insert("LEFT_CTRL", (*KEY_RIGHTCTRL, KeyModifierFlags::new()));
        m.insert("LEFT_SHIFT", (*KEY_LEFTSHIFT, KeyModifierFlags::new()));
        m.insert("META", (*KEY_LEFTMETA, KeyModifierFlags::new()));
        m.insert("PAGE_DOWN", (*KEY_PAGEDOWN, KeyModifierFlags::new()));
        m.insert("PAGE_UP", (*KEY_PAGEUP, KeyModifierFlags::new()));
        m.insert("RIGHT_ALT", (*KEY_RIGHTALT, KeyModifierFlags::new()));
        m.insert("RIGHT_BRACE", (*KEY_RIGHTBRACE, KeyModifierFlags::new()));
        m.insert("LEFT_BRACE", (*KEY_LEFTBRACE, KeyModifierFlags::new()));
        m.insert("RIGHT_CTRL", (*KEY_RIGHTCTRL, KeyModifierFlags::new()));
        m.insert("RIGHT_SHIFT", (*KEY_RIGHTSHIFT, KeyModifierFlags::new()));
        m.insert("SHIFT", (*KEY_LEFTSHIFT, KeyModifierFlags::new()));
        m
    };
}



