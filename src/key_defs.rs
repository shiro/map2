use evdev_rs::enums::{EV_SYN, EventCode, EventType};
use evdev_rs::TimeVal;
use tap::Tap;

use crate::*;

pub const INPUT_EV_DUMMY_TIME: TimeVal = TimeVal { tv_sec: 0, tv_usec: 0 };

pub static SYN_REPORT: EvdevInputEvent = EvdevInputEvent { event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT), value: 0, time: INPUT_EV_DUMMY_TIME };

lazy_static! {
pub static ref KEY_LEFT_META: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTMETA").unwrap();
pub static ref KEY_RIGHT_META: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTMETA").unwrap();
pub static ref KEY_LEFT_ALT: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTALT").unwrap();
pub static ref KEY_RIGHT_ALT: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTALT").unwrap();
pub static ref KEY_LEFT_SHIFT: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTSHIFT").unwrap();
pub static ref KEY_RIGHT_SHIFT: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTSHIFT").unwrap();
pub static ref KEY_LEFT_CTRL: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTCTRL").unwrap();
pub static ref KEY_RIGHT_CTRL: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTCTRL").unwrap();
pub static ref KEY_ENTER: Key = Key::from_str(&EventType::EV_KEY, "KEY_ENTER").unwrap();
pub static ref KEY_ESC: Key = Key::from_str(&EventType::EV_KEY, "KEY_ESC").unwrap();
pub static ref KEY_TAB: Key = Key::from_str(&EventType::EV_KEY, "KEY_TAB").unwrap();
pub static ref KEY_SPACE: Key = Key::from_str(&EventType::EV_KEY, "KEY_SPACE").unwrap();
pub static ref KEY_MINUS: Key = Key::from_str(&EventType::EV_KEY, "KEY_MINUS").unwrap();
pub static ref KEY_SLASH: Key = Key::from_str(&EventType::EV_KEY, "KEY_SLASH").unwrap();
pub static ref KEY_CAPSLOCK: Key = Key::from_str(&EventType::EV_KEY, "KEY_CAPSLOCK").unwrap();
pub static ref KEY_LEFT: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFT").unwrap();
pub static ref KEY_RIGHT: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHT").unwrap();
pub static ref KEY_UP: Key = Key::from_str(&EventType::EV_KEY, "KEY_UP").unwrap();
pub static ref KEY_DOWN: Key = Key::from_str(&EventType::EV_KEY, "KEY_DOWN").unwrap();
pub static ref KEY_F4: Key = Key::from_str(&EventType::EV_KEY, "KEY_F4").unwrap();
pub static ref KEY_F5: Key = Key::from_str(&EventType::EV_KEY, "KEY_F5").unwrap();
pub static ref KEY_A: Key = Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap();
pub static ref KEY_B: Key = Key::from_str(&EventType::EV_KEY, "KEY_B").unwrap();
pub static ref KEY_C: Key = Key::from_str(&EventType::EV_KEY, "KEY_C").unwrap();
pub static ref KEY_D: Key = Key::from_str(&EventType::EV_KEY, "KEY_D").unwrap();
pub static ref KEY_E: Key = Key::from_str(&EventType::EV_KEY, "KEY_E").unwrap();
pub static ref KEY_F: Key = Key::from_str(&EventType::EV_KEY, "KEY_F").unwrap();
pub static ref KEY_G: Key = Key::from_str(&EventType::EV_KEY, "KEY_G").unwrap();
pub static ref KEY_H: Key = Key::from_str(&EventType::EV_KEY, "KEY_H").unwrap();
pub static ref KEY_I: Key = Key::from_str(&EventType::EV_KEY, "KEY_I").unwrap();
pub static ref KEY_J: Key = Key::from_str(&EventType::EV_KEY, "KEY_J").unwrap();
pub static ref KEY_K: Key = Key::from_str(&EventType::EV_KEY, "KEY_K").unwrap();
pub static ref KEY_L: Key = Key::from_str(&EventType::EV_KEY, "KEY_L").unwrap();
pub static ref KEY_M: Key = Key::from_str(&EventType::EV_KEY, "KEY_M").unwrap();
pub static ref KEY_N: Key = Key::from_str(&EventType::EV_KEY, "KEY_N").unwrap();
pub static ref KEY_O: Key = Key::from_str(&EventType::EV_KEY, "KEY_O").unwrap();
pub static ref KEY_P: Key = Key::from_str(&EventType::EV_KEY, "KEY_P").unwrap();
pub static ref KEY_Q: Key = Key::from_str(&EventType::EV_KEY, "KEY_Q").unwrap();
pub static ref KEY_R: Key = Key::from_str(&EventType::EV_KEY, "KEY_R").unwrap();
pub static ref KEY_S: Key = Key::from_str(&EventType::EV_KEY, "KEY_S").unwrap();
pub static ref KEY_T: Key = Key::from_str(&EventType::EV_KEY, "KEY_T").unwrap();
pub static ref KEY_U: Key = Key::from_str(&EventType::EV_KEY, "KEY_U").unwrap();
pub static ref KEY_V: Key = Key::from_str(&EventType::EV_KEY, "KEY_V").unwrap();
pub static ref KEY_W: Key = Key::from_str(&EventType::EV_KEY, "KEY_W").unwrap();
pub static ref KEY_X: Key = Key::from_str(&EventType::EV_KEY, "KEY_X").unwrap();
pub static ref KEY_Y: Key = Key::from_str(&EventType::EV_KEY, "KEY_Y").unwrap();
pub static ref KEY_Z: Key = Key::from_str(&EventType::EV_KEY, "KEY_Z").unwrap();
pub static ref KEY_KPD0: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP0").unwrap();
pub static ref KEY_KPD1: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP1").unwrap();
pub static ref KEY_KPD2: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP2").unwrap();
pub static ref KEY_KPD3: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP3").unwrap();
pub static ref KEY_KPD4: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP4").unwrap();
pub static ref KEY_KPD5: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP5").unwrap();
pub static ref KEY_KPD6: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP6").unwrap();
pub static ref KEY_KPD7: Key = Key::from_str(&EventType::EV_KEY, "KEY_KP7").unwrap();
pub static ref KEY_BACKSPACE: Key = Key::from_str(&EventType::EV_KEY, "KEY_BACKSPACE").unwrap();
pub static ref KEY_GRAVE: Key = Key::from_str(&EventType::EV_KEY, "KEY_GRAVE").unwrap();
pub static ref KEY_LEFTSHIFT: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTSHIFT").unwrap();
pub static ref KEY_RIGHTSHIFT: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTSHIFT").unwrap();
pub static ref KEY_PAGEUP: Key = Key::from_str(&EventType::EV_KEY, "KEY_PAGEUP").unwrap();
pub static ref KEY_PAGEDOWN: Key = Key::from_str(&EventType::EV_KEY, "KEY_PAGEDOWN").unwrap();
pub static ref KEY_LEFTBRACE: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTBRACE").unwrap();
pub static ref KEY_RIGHTBRACE: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTBRACE").unwrap();
pub static ref KEY_SEMICOLON: Key = Key::from_str(&EventType::EV_KEY, "KEY_SEMICOLON").unwrap();
pub static ref KEY_6: Key = Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap();
pub static ref BTN_LEFT: Key = Key::from_str(&EventType::EV_KEY, "BTN_LEFT").unwrap();
pub static ref BTN_RIGHT: Key = Key::from_str(&EventType::EV_KEY, "BTN_RIGHT").unwrap();
pub static ref BTN_MIDDLE: Key = Key::from_str(&EventType::EV_KEY, "BTN_MIDDLE").unwrap();
}


lazy_static! {
    pub(crate) static ref KEY_ALIAS_TABLE: HashMap<&'static str, (Key, KeyModifierFlags)> = {
        let mut m = HashMap::new();
        m.insert("BTN_LEFT", (*BTN_LEFT, KeyModifierFlags::new()));
        m.insert("BTN_RIGHT", (*BTN_RIGHT, KeyModifierFlags::new()));
        m.insert("BTN_MIDDLE", (*BTN_MIDDLE, KeyModifierFlags::new()));
        m.insert("SPACE", (*KEY_SPACE, KeyModifierFlags::new()));
        m.insert("BACKSPACE", (*KEY_BACKSPACE, KeyModifierFlags::new()));
        m.insert("SHIFT", (*KEY_LEFTSHIFT, KeyModifierFlags::new()));
        m.insert("LEFT_SHIFT", (*KEY_LEFTSHIFT, KeyModifierFlags::new()));
        m.insert("RIGHT_SHIFT", (*KEY_RIGHTSHIFT, KeyModifierFlags::new()));
        m.insert("LEFT", (*KEY_LEFT, KeyModifierFlags::new()));
        m.insert("RIGHT", (*KEY_RIGHT, KeyModifierFlags::new()));
        m.insert("UP", (*KEY_UP, KeyModifierFlags::new()));
        m.insert("DOWN", (*KEY_DOWN, KeyModifierFlags::new()));
        m.insert("PAGE_UP", (*KEY_PAGEUP, KeyModifierFlags::new()));
        m.insert("PAGE_DOWN", (*KEY_PAGEDOWN, KeyModifierFlags::new()));
        m.insert("TAB", (*KEY_TAB, KeyModifierFlags::new()));
        m.insert("ALT", (Key::from_str(&EventType::EV_KEY, "KEY_LEFTALT").unwrap(), KeyModifierFlags::new()));
        m.insert("LEFT_ALT", (Key::from_str(&EventType::EV_KEY, "KEY_LEFTALT").unwrap(), KeyModifierFlags::new()));
        m.insert("RIGHT_ALT", (Key::from_str(&EventType::EV_KEY, "KEY_RIGHTALT").unwrap(), KeyModifierFlags::new()));
        m.insert("META", (Key::from_str(&EventType::EV_KEY, "KEY_LEFTMETA").unwrap(), KeyModifierFlags::new()));
        m.insert("LEFT_ALT", (Key::from_str(&EventType::EV_KEY, "KEY_LEFTALT").unwrap(), KeyModifierFlags::new()));
        m.insert("RIGHT_ALT", (Key::from_str(&EventType::EV_KEY, "KEY_RIGHTALT").unwrap(), KeyModifierFlags::new()));
        m.insert("CTRL", (Key::from_str(&EventType::EV_KEY, "KEY_LEFTCTRL").unwrap(), KeyModifierFlags::new()));
        m.insert("LEFT_CTRL", (Key::from_str(&EventType::EV_KEY, "KEY_LEFTCTRL").unwrap(), KeyModifierFlags::new()));
        m.insert("RIGHT_CTRL", (Key::from_str(&EventType::EV_KEY, "KEY_RIGHTCTRL").unwrap(), KeyModifierFlags::new()));
        m.insert("CAPSLOCK", (Key::from_str(&EventType::EV_KEY, "KEY_CAPSLOCK").unwrap(), KeyModifierFlags::new()));
        m
    };
}



