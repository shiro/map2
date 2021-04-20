use crate::*;
// use input_linux_sys;
use input_linux_sys::*;
use crate::block_ext::ExprVecExt;

#[allow(non_camel_case_types)]
pub type time_t = i64;
#[allow(non_camel_case_types)]
pub type suseconds_t = i64;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
#[allow(non_camel_case_types)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct input_event {
    pub time: timeval,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

impl PartialEq for input_event {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code &&
            self.type_ == other.type_ &&
            self.value == other.value
    }
}

impl PartialEq<Key> for input_event {
    fn eq(&self, other: &Key) -> bool {
        self.code as i32 == other.code &&
            self.value == other.key_type
    }
}

impl Eq for input_event {}

pub(crate) const fn make_key(code: i32) -> Key { Key { key_type: input_linux_sys::EV_KEY, code } }

pub const INPUT_EV_DUMMY_TIME: timeval = timeval { tv_sec: 0, tv_usec: 0 };
pub const INPUT_EV_SYN: input_event = input_event { type_: EV_SYN as u16, code: SYN_REPORT as u16, value: 0, time: INPUT_EV_DUMMY_TIME };


pub static KEY_MOUSE5: Key = make_key(277);
pub static KEY_MOUSE6: Key = make_key(278);
pub static KEY_MOUSE7: Key = make_key(279);
pub static KEY_MOUSE8: Key = make_key(280);
pub static KEY_MOUSE9: Key = make_key(281);
pub static KEY_MOUSE10: Key = make_key(282);
pub static KEY_MOUSE11: Key = make_key(283);
pub static KEY_MOUSE12: Key = make_key(284);
pub static KEY_LEFT_META: Key = make_key(input_linux_sys::KEY_LEFTMETA);
pub static KEY_LEFT_ALT: Key = make_key(input_linux_sys::KEY_LEFTALT);
pub static KEY_RIGHT_ALT: Key = make_key(input_linux_sys::KEY_RIGHTALT);
pub static KEY_LEFT_SHIFT: Key = make_key(input_linux_sys::KEY_LEFTSHIFT);
pub static KEY_LEFT_CTRL: Key = make_key(input_linux_sys::KEY_LEFTCTRL);
pub static KEY_ENTER: Key = make_key(input_linux_sys::KEY_ENTER);
pub static KEY_ESC: Key = make_key(input_linux_sys::KEY_ESC);
pub static KEY_TAB: Key = make_key(input_linux_sys::KEY_TAB);
pub static KEY_SPACE: Key = make_key(input_linux_sys::KEY_SPACE);
pub static KEY_MINUS: Key = make_key(input_linux_sys::KEY_MINUS);
pub static KEY_SLASH: Key = make_key(input_linux_sys::KEY_SLASH);
pub static KEY_CAPSLOCK: Key = make_key(input_linux_sys::KEY_CAPSLOCK);
pub static KEY_LEFT: Key = make_key(input_linux_sys::KEY_LEFT);
pub static KEY_RIGHT: Key = make_key(input_linux_sys::KEY_RIGHT);
pub static KEY_UP: Key = make_key(input_linux_sys::KEY_UP);
pub static KEY_DOWN: Key = make_key(input_linux_sys::KEY_DOWN);
pub static KEY_F4: Key = make_key(input_linux_sys::KEY_F4);
pub static KEY_F5: Key = make_key(input_linux_sys::KEY_F5);
pub static KEY_A: Key = make_key(input_linux_sys::KEY_A);
pub static KEY_B: Key = make_key(input_linux_sys::KEY_B);
pub static KEY_C: Key = make_key(input_linux_sys::KEY_C);
pub static KEY_D: Key = make_key(input_linux_sys::KEY_D);
pub static KEY_E: Key = make_key(input_linux_sys::KEY_E);
pub static KEY_F: Key = make_key(input_linux_sys::KEY_F);
pub static KEY_G: Key = make_key(input_linux_sys::KEY_G);
pub static KEY_H: Key = make_key(input_linux_sys::KEY_H);
pub static KEY_I: Key = make_key(input_linux_sys::KEY_I);
pub static KEY_J: Key = make_key(input_linux_sys::KEY_J);
pub static KEY_K: Key = make_key(input_linux_sys::KEY_K);
pub static KEY_L: Key = make_key(input_linux_sys::KEY_L);
pub static KEY_M: Key = make_key(input_linux_sys::KEY_M);
pub static KEY_N: Key = make_key(input_linux_sys::KEY_N);
pub static KEY_O: Key = make_key(input_linux_sys::KEY_O);
pub static KEY_P: Key = make_key(input_linux_sys::KEY_P);
pub static KEY_Q: Key = make_key(input_linux_sys::KEY_Q);
pub static KEY_R: Key = make_key(input_linux_sys::KEY_R);
pub static KEY_S: Key = make_key(input_linux_sys::KEY_S);
pub static KEY_T: Key = make_key(input_linux_sys::KEY_T);
pub static KEY_U: Key = make_key(input_linux_sys::KEY_U);
pub static KEY_V: Key = make_key(input_linux_sys::KEY_V);
pub static KEY_W: Key = make_key(input_linux_sys::KEY_W);
pub static KEY_X: Key = make_key(input_linux_sys::KEY_X);
pub static KEY_Y: Key = make_key(input_linux_sys::KEY_Y);
pub static KEY_Z: Key = make_key(input_linux_sys::KEY_Z);
pub static KEY_KPD0: Key = make_key(input_linux_sys::KEY_KP0);
pub static KEY_KPD1: Key = make_key(input_linux_sys::KEY_KP1);
pub static KEY_KPD2: Key = make_key(input_linux_sys::KEY_KP2);
pub static KEY_KPD3: Key = make_key(input_linux_sys::KEY_KP3);
pub static KEY_KPD4: Key = make_key(input_linux_sys::KEY_KP4);
pub static KEY_KPD5: Key = make_key(input_linux_sys::KEY_KP5);
pub static KEY_KPD6: Key = make_key(input_linux_sys::KEY_KP6);
pub static KEY_KPD7: Key = make_key(input_linux_sys::KEY_KP7);


pub struct InputEvGroup {
    pub up: input_event,
    pub down: input_event,
    pub repeat: input_event,
}

impl InputEvGroup {
    pub const fn new(code: i32) -> Self {
        InputEvGroup {
            up: input_event { type_: EV_KEY as u16, code: code as u16, value: 0, time: INPUT_EV_DUMMY_TIME },
            down: input_event { type_: EV_KEY as u16, code: code as u16, value: 1, time: INPUT_EV_DUMMY_TIME },
            repeat: input_event { type_: EV_KEY as u16, code: code as u16, value: 2, time: INPUT_EV_DUMMY_TIME },
        }
    }
    pub fn to_key(&self) -> Key {
        make_key(self.up.code as i32)
    }
}

// region key codes
pub const INPUT_EV_TAB: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_TAB);
pub const INPUT_EV_LEFTMETA: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTMETA);
pub const INPUT_EV_RIGHTMETA: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_RIGHTMETA);
pub const INPUT_EV_SHIFT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTSHIFT);
pub const INPUT_EV_LEFTALT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTALT);
pub const INPUT_EV_RIGHTALT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_RIGHTALT);
pub const INPUT_EV_CAPSLOCK: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_CAPSLOCK);
pub const INPUT_EV_LEFTCTRL: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFTCTRL);
pub const INPUT_EV_ESC: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_ESC);
pub const INPUT_EV_H: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_H);
pub const INPUT_EV_J: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_J);
pub const INPUT_EV_K: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_K);
pub const INPUT_EV_L: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_L);
pub const INPUT_EV_ARROW_LEFT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_LEFT);
pub const INPUT_EV_ARROW_DOWN: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_DOWN);
pub const INPUT_EV_ARROW_RIGHT: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_RIGHT);
pub const INPUT_EV_ARROW_UP: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_UP);
pub const INPUT_EV_F8: InputEvGroup = InputEvGroup::new(input_linux_sys::KEY_F8);
pub const INPUT_EV_MOUSE5: InputEvGroup = InputEvGroup::new(277);
pub const INPUT_EV_MOUSE6: InputEvGroup = InputEvGroup::new(277);
pub const INPUT_EV_MOUSE7: InputEvGroup = InputEvGroup::new(279);
pub const INPUT_EV_MOUSE8: InputEvGroup = InputEvGroup::new(280);
pub const INPUT_EV_MOUSE9: InputEvGroup = InputEvGroup::new(281);
pub const INPUT_EV_MOUSE10: InputEvGroup = InputEvGroup::new(282);
pub const INPUT_EV_MOUSE11: InputEvGroup = InputEvGroup::new(283);
pub const INPUT_EV_MOUSE12: InputEvGroup = InputEvGroup::new(284);
// const WHEEL: input_event = input_event { type_: EV_REL as u16, code: REL_WHEEL as u16, value: 0, time: DUMMY_TIME };
// endregion


type KEYCODE = i32;

trait KeycodeExt {
    fn to_key(&self) -> Key;
}

impl KeycodeExt for KEYCODE {
    fn to_key(&self) -> Key { Key { key_type: EV_KEY, code: *self } }
}

lazy_static! {
    pub(crate) static ref KEY_LOOKUP: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        m.insert("enter", KEY_ENTER);
        m.insert("esc", KEY_ESC);
        m.insert("ctrl", KEY_LEFT_CTRL);
        m.insert("shift", KEY_LEFT_SHIFT);
        m.insert(" ", KEY_SPACE);
        m.insert("-", KEY_MINUS);
        m.insert("/", KEY_SLASH);
        m.insert("a", KEY_A);
        m.insert("b", KEY_B);
        m.insert("c", KEY_C);
        m.insert("d", KEY_D);
        m.insert("e", KEY_E);
        m.insert("f", KEY_F);
        m.insert("g", KEY_G);
        m.insert("h", KEY_H);
        m.insert("i", KEY_I);
        m.insert("j", KEY_J);
        m.insert("k", KEY_K);
        m.insert("l", KEY_L);
        m.insert("m", KEY_M);
        m.insert("n", KEY_N);
        m.insert("o", KEY_O);
        m.insert("p", KEY_P);
        m.insert("q", KEY_Q);
        m.insert("r", KEY_R);
        m.insert("s", KEY_S);
        m.insert("t", KEY_T);
        m.insert("u", KEY_U);
        m.insert("v", KEY_V);
        m.insert("w", KEY_W);
        m.insert("x", KEY_X);
        m.insert("y", KEY_Y);
        m.insert("z", KEY_Z);
        m
    };
}


lazy_static! {
    pub(crate) static ref KEY_SEQ_LOOKUP: HashMap<&'static str, Vec<Expr>> = {
        let mut m = HashMap::new();
        m.insert("enter", vec![].append_click(KEY_ENTER));
        m.insert("esc", vec![].append_click(KEY_ESC));
        m.insert("ctrl", vec![].append_click(KEY_LEFT_CTRL));
        m.insert("ctrl down", vec![] .append_action(KeyAction::new(KEY_LEFT_CTRL, TYPE_DOWN)));
        m.insert("ctrl up", vec![] .append_action(KeyAction::new(KEY_LEFT_CTRL, TYPE_UP)));
        m.insert("shift", vec![].append_click(KEY_LEFT_SHIFT));
        m.insert(" ", vec![].append_click(KEY_SPACE));
        m.insert("-", vec![].append_click(KEY_MINUS));
        m.insert("/", vec![].append_click(KEY_SLASH));
        m.insert("a", vec![].append_click(KEY_A));
        m.insert("b", vec![].append_click(KEY_B));
        m.insert("c", vec![].append_click(KEY_C));
        m.insert("d", vec![].append_click(KEY_D));
        m.insert("e", vec![].append_click(KEY_E));
        m.insert("f", vec![].append_click(KEY_F));
        m.insert("g", vec![].append_click(KEY_G));
        m.insert("h", vec![].append_click(KEY_H));
        m.insert("i", vec![].append_click(KEY_I));
        m.insert("j", vec![].append_click(KEY_J));
        m.insert("k", vec![].append_click(KEY_K));
        m.insert("l", vec![].append_click(KEY_L));
        m.insert("m", vec![].append_click(KEY_M));
        m.insert("n", vec![].append_click(KEY_N));
        m.insert("o", vec![].append_click(KEY_O));
        m.insert("p", vec![].append_click(KEY_P));
        m.insert("q", vec![].append_click(KEY_Q));
        m.insert("r", vec![].append_click(KEY_R));
        m.insert("s", vec![].append_click(KEY_S));
        m.insert("t", vec![].append_click(KEY_T));
        m.insert("u", vec![].append_click(KEY_U));
        m.insert("v", vec![].append_click(KEY_V));
        m.insert("w", vec![].append_click(KEY_W));
        m.insert("x", vec![].append_click(KEY_X));
        m.insert("y", vec![].append_click(KEY_Y));
        m.insert("z", vec![].append_click(KEY_Z));
        m.insert("V", vec![]
            .append_action(KeyAction::new(KEY_LEFT_SHIFT, TYPE_DOWN))
            .append_click(KEY_V)
            .append_action(KeyAction::new(KEY_LEFT_SHIFT, TYPE_UP))
        );
        m
    };
}
