use evdev_rs::enums::{EV_SYN, EventCode};
use evdev_rs::TimeVal;

use crate::*;

pub const INPUT_EV_DUMMY_TIME: TimeVal = TimeVal { tv_sec: 0, tv_usec: 0 };

pub static SYN_REPORT: EvdevInputEvent = EvdevInputEvent { event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT), value: 0, time: INPUT_EV_DUMMY_TIME };


lazy_static! {
    pub(crate) static ref KEY_ALIAS_TABLE: HashMap<&'static str, (Key, KeyModifierFlags)> = {
        use evdev_rs::enums::EV_KEY::*;

        let mut m = HashMap::new();
        m.insert("ALT", (KEY_LEFTALT.into(), KeyModifierFlags::new()));
        m.insert("CTRL", (KEY_LEFTCTRL.into(), KeyModifierFlags::new()));
        m.insert("ESCAPE", (KEY_ESC.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_0", (KEY_KP0.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_1", (KEY_KP1.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_2", (KEY_KP2.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_3", (KEY_KP3.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_4", (KEY_KP4.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_5", (KEY_KP5.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_6", (KEY_KP6.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_7", (KEY_KP7.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_8", (KEY_KP8.into(), KeyModifierFlags::new()));
        m.insert("KEYPAD_9", (KEY_KP9.into(), KeyModifierFlags::new()));
        m.insert("LEFT_ALT", (KEY_LEFTALT.into(), KeyModifierFlags::new()));
        m.insert("LEFT_CTRL", (KEY_RIGHTCTRL.into(), KeyModifierFlags::new()));
        m.insert("LEFT_SHIFT", (KEY_LEFTSHIFT.into(), KeyModifierFlags::new()));
        m.insert("META", (KEY_LEFTMETA.into(), KeyModifierFlags::new()));
        m.insert("PAGE_DOWN", (KEY_PAGEDOWN.into(), KeyModifierFlags::new()));
        m.insert("PAGE_UP", (KEY_PAGEUP.into(), KeyModifierFlags::new()));
        m.insert("RIGHT_ALT", (KEY_RIGHTALT.into(), KeyModifierFlags::new()));
        m.insert("RIGHT_BRACE", (KEY_RIGHTBRACE.into(), KeyModifierFlags::new()));
        m.insert("LEFT_BRACE", (KEY_LEFTBRACE.into(), KeyModifierFlags::new()));
        m.insert("RIGHT_CTRL", (KEY_RIGHTCTRL.into(), KeyModifierFlags::new()));
        m.insert("RIGHT_SHIFT", (KEY_RIGHTSHIFT.into(), KeyModifierFlags::new()));
        m.insert("SHIFT", (KEY_LEFTSHIFT.into(), KeyModifierFlags::new()));
        m
    };
}



