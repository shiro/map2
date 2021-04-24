use evdev_rs::enums::{EventType, EventCode, EV_SYN};
use evdev_rs::TimeVal;

use crate::*;
use crate::block_ext::ExprVecExt;

pub(crate) const fn make_key(event_code: EventCode) -> Key { Key { event_code } }

pub const INPUT_EV_DUMMY_TIME: TimeVal = TimeVal { tv_sec: 0, tv_usec: 0 };
// pub const INPUT_EV_SYN: input_event = InputEvent { event_code: EventCode::EV_REP(), value: 0, time: INPUT_EV_DUMMY_TIME };


// pub static KEY_MOUSE5: Key = make_key(277);
// pub static KEY_MOUSE6: Key = make_key(278);
// pub static KEY_MOUSE7: Key = make_key(279);
// pub static KEY_MOUSE8: Key = make_key(280);
// pub static KEY_MOUSE9: Key = make_key(281);
// pub static KEY_MOUSE10: Key = make_key(282);
// pub static KEY_MOUSE11: Key = make_key(283);
// pub static KEY_MOUSE12: Key = make_key(284);

pub static SYN_REPORT: InputEvent = InputEvent { event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT), value: 0, time: INPUT_EV_DUMMY_TIME };

lazy_static! {
pub static ref KEY_LEFT_META: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTMETA").unwrap();
pub static ref KEY_LEFT_ALT: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTALT").unwrap();
pub static ref KEY_RIGHT_ALT: Key = Key::from_str(&EventType::EV_KEY, "KEY_RIGHTALT").unwrap();
pub static ref KEY_LEFT_SHIFT: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTSHIFT").unwrap();
pub static ref KEY_LEFT_CTRL: Key = Key::from_str(&EventType::EV_KEY, "KEY_LEFTCTRL").unwrap();
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
}


pub struct InputEvGroup {
    pub up: InputEvent,
    pub down: InputEvent,
    pub repeat: InputEvent,
}

impl InputEvGroup {
    pub const fn new(event_code: EventCode) -> Self {
        InputEvGroup {
            up: InputEvent { event_code, value: 0, time: INPUT_EV_DUMMY_TIME },
            down: InputEvent { event_code, value: 1, time: INPUT_EV_DUMMY_TIME },
            repeat: InputEvent { event_code, value: 2, time: INPUT_EV_DUMMY_TIME },
        }
    }
    pub fn to_key(&self) -> Key {
        make_key(self.up.event_code)
    }
}
// region key codes
// pub const INPUT_EV_TAB: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_TAB);
// pub const INPUT_EV_LEFTMETA: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTMETA);
// pub const INPUT_EV_RIGHTMETA: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_RIGHTMETA);
// pub const INPUT_EV_SHIFT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTSHIFT);
// pub const INPUT_EV_LEFTALT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTALT);
// pub const INPUT_EV_RIGHTALT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_RIGHTALT);
// pub const INPUT_EV_CAPSLOCK: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_CAPSLOCK);
// pub const INPUT_EV_LEFTCTRL: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTCTRL);
// pub const INPUT_EV_ESC: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_ESC);
// pub const INPUT_EV_H: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_H);
// pub const INPUT_EV_J: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_J);
// pub const INPUT_EV_K: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_K);
// pub const INPUT_EV_L: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_L);
// pub const INPUT_EV_ARROW_LEFT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFT);
// pub const INPUT_EV_ARROW_DOWN: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_DOWN);
// pub const INPUT_EV_ARROW_RIGHT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_RIGHT);
// pub const INPUT_EV_ARROW_UP: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_UP);
// pub const INPUT_EV_F8: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_F8);
// pub const INPUT_EV_MOUSE5: InputEvGroup = InputEvGroup::new(277);
// pub const INPUT_EV_MOUSE6: InputEvGroup = InputEvGroup::new(277);
// pub const INPUT_EV_MOUSE7: InputEvGroup = InputEvGroup::new(279);
// pub const INPUT_EV_MOUSE8: InputEvGroup = InputEvGroup::new(280);
// pub const INPUT_EV_MOUSE9: InputEvGroup = InputEvGroup::new(281);
// pub const INPUT_EV_MOUSE10: InputEvGroup = InputEvGroup::new(282);
// pub const INPUT_EV_MOUSE11: InputEvGroup = InputEvGroup::new(283);
// pub const INPUT_EV_MOUSE12: InputEvGroup = InputEvGroup::new(284);
// const WHEEL: input_event = input_event { type_: EV_REL as u16, code: REL_WHEEL as u16, value: 0, time: DUMMY_TIME };
// endregion


// type KEYCODE = i32;

// trait KeycodeExt {
//     fn to_key(&self) -> Key;
// }
//
// impl KeycodeExt for KEYCODE {
//     fn to_key(&self) -> Key { Key { key_type: EV_KEY, code: *self } }
// }

lazy_static! {
    pub(crate) static ref KEY_LOOKUP: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        m.insert("a", Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap());

        // m.insert("mouse5", KEY_MOUSE5);
        // m.insert("mouse6", KEY_MOUSE6);
        // m.insert("mouse7", KEY_MOUSE7);
        // m.insert("mouse8", KEY_MOUSE8);
        // m.insert("mouse9", KEY_MOUSE9);
        // m.insert("mouse10", KEY_MOUSE10);
        // m.insert("mouse11", KEY_MOUSE11);
        // m.insert("mouse12", KEY_MOUSE12);
        // m.insert("enter", KEY_ENTER);
        // m.insert("esc", KEY_ESC);
        // m.insert("ctrl", KEY_LEFT_CTRL);
        // m.insert("shift", KEY_LEFT_SHIFT);
        // m.insert(" ", KEY_SPACE);
        // m.insert("-", KEY_MINUS);
        // m.insert("/", KEY_SLASH);
        // m.insert("a", KEY_A);
        // m.insert("b", KEY_B);
        // m.insert("c", KEY_C);
        // m.insert("d", KEY_D);
        // m.insert("e", KEY_E);
        // m.insert("f", KEY_F);
        // m.insert("g", KEY_G);
        // m.insert("h", KEY_H);
        // m.insert("i", KEY_I);
        // m.insert("j", KEY_J);
        // m.insert("k", KEY_K);
        // m.insert("l", KEY_L);
        // m.insert("m", KEY_M);
        // m.insert("n", KEY_N);
        // m.insert("o", KEY_O);
        // m.insert("p", KEY_P);
        // m.insert("q", KEY_Q);
        // m.insert("r", KEY_R);
        // m.insert("s", KEY_S);
        // m.insert("t", KEY_T);
        // m.insert("u", KEY_U);
        // m.insert("v", KEY_V);
        // m.insert("w", KEY_W);
        // m.insert("x", KEY_X);
        // m.insert("y", KEY_Y);
        // m.insert("z", KEY_Z);
        m
    };
}


lazy_static! {
    pub(crate) static ref KEY_SEQ_LOOKUP: HashMap<&'static str, Vec<Expr>> = {
        let mut m = HashMap::new();
        m.insert("enter", vec![].append_click(*KEY_ENTER));
        m.insert("esc", vec![].append_click(*KEY_ESC));
        m.insert("ctrl", vec![].append_click(*KEY_LEFT_CTRL));
        m.insert("ctrl down", vec![] .append_action(KeyAction::new(*KEY_LEFT_CTRL, TYPE_DOWN)));
        m.insert("ctrl up", vec![] .append_action(KeyAction::new(*KEY_LEFT_CTRL, TYPE_UP)));
        m.insert("shift", vec![].append_click(*KEY_LEFT_SHIFT));
        m.insert(" ", vec![].append_click(*KEY_SPACE));
        m.insert("-", vec![].append_click(*KEY_MINUS));
        m.insert("/", vec![].append_click(*KEY_SLASH));
        m.insert("a", vec![].append_click(*KEY_A));
        m.insert("b", vec![].append_click(*KEY_B));
        m.insert("c", vec![].append_click(*KEY_C));
        m.insert("d", vec![].append_click(*KEY_D));
        m.insert("e", vec![].append_click(*KEY_E));
        m.insert("f", vec![].append_click(*KEY_F));
        m.insert("g", vec![].append_click(*KEY_G));
        m.insert("h", vec![].append_click(*KEY_H));
        m.insert("i", vec![].append_click(*KEY_I));
        m.insert("j", vec![].append_click(*KEY_J));
        m.insert("k", vec![].append_click(*KEY_K));
        m.insert("l", vec![].append_click(*KEY_L));
        m.insert("m", vec![].append_click(*KEY_M));
        m.insert("n", vec![].append_click(*KEY_N));
        m.insert("o", vec![].append_click(*KEY_O));
        m.insert("p", vec![].append_click(*KEY_P));
        m.insert("q", vec![].append_click(*KEY_Q));
        m.insert("r", vec![].append_click(*KEY_R));
        m.insert("s", vec![].append_click(*KEY_S));
        m.insert("t", vec![].append_click(*KEY_T));
        m.insert("u", vec![].append_click(*KEY_U));
        m.insert("v", vec![].append_click(*KEY_V));
        m.insert("w", vec![].append_click(*KEY_W));
        m.insert("x", vec![].append_click(*KEY_X));
        m.insert("y", vec![].append_click(*KEY_Y));
        m.insert("z", vec![].append_click(*KEY_Z));
        m.insert("V", vec![]
            .append_action(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_DOWN))
            .append_click(*KEY_V)
            .append_action(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_UP))
        );
        m
    };
}
