#![feature(type_ascription)]
#![feature(async_closure)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(label_break_value)]

#[macro_use]
extern crate lazy_static;

use std::{io, mem, thread, time};
use std::io::{stdout, Write};
use std::sync::{Arc};

use anyhow::Result;
use input_linux_sys::*;
use nom::lib::std::collections::HashMap;
use tokio::prelude::*;
use tokio::task;

use crate::x11::{x11_initialize, x11_test};
use crate::x11::ActiveWindowResult;
use tokio::sync::mpsc::Sender;

mod tab_mod;
mod caps_mod;
mod rightalt_mod;
mod x11;

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

impl Eq for input_event {}

struct IgnoreList(Vec<KeyAction>);

impl IgnoreList {
    pub fn new() -> Self { IgnoreList(Default::default()) }

    fn is_ignored(&self, key: &KeyAction) -> bool {
        match self.0.iter().position(|x| x == key) {
            None => false,
            Some(_) => true
        }
    }

    fn unignore(&mut self, key: &KeyAction) {
        if let Some(pos) = self.0.iter().position(|x| x == key) {
            self.0.remove(pos);
        }
    }

    fn ignore(&mut self, key: &KeyAction) {
        if let None = self.0.iter().position(|x| x == key) {
            self.0.push(*key);
        }
    }
}

pub struct State {
    tab_is_down: bool,
    capslock_is_down: bool,
    leftcontrol_is_down: bool,
    shift_is_down: bool,
    meta_is_down: bool,
    leftalt_is_down: bool,
    right_alt_is_down: bool,

    disable_alt_mod: bool,

    ignore_list: IgnoreList,
    mappings: KeyMappings,

    active_window: Option<ActiveWindowResult>,
}

unsafe fn any_as_u8_slice_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    ::std::slice::from_raw_parts_mut(
        (p as *const T) as *mut u8,
        ::std::mem::size_of::<T>(),
    )
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

fn print_event(ev: &input_event) {
    unsafe {
        stdout().write_all(any_as_u8_slice(ev)).unwrap();
        // if ev.type_ == EV_KEY as u16 {
        //     log_msg(format!("{:?}", ev).as_str());
        // }
        stdout().flush().unwrap();
    }
}

fn equal(ev1: &input_event, ev2: &input_event) -> bool {
    ev1.code == ev2.code &&
        ev1.type_ == ev2.type_ &&
        ev1.value == ev2.value
}


// region key codes
const DUMMY_TIME: timeval = timeval { tv_sec: 0, tv_usec: 0 };

const TAB_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_TAB as u16, value: 0, time: DUMMY_TIME };
const TAB_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_TAB as u16, value: 1, time: DUMMY_TIME };
const TAB_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_TAB as u16, value: 2, time: DUMMY_TIME };

const META_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTMETA as u16, value: 0, time: DUMMY_TIME };
const META_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTMETA as u16, value: 1, time: DUMMY_TIME };
const META_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTMETA as u16, value: 2, time: DUMMY_TIME };

const SHIFT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTSHIFT as u16, value: 0, time: DUMMY_TIME };
const SHIFT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTSHIFT as u16, value: 1, time: DUMMY_TIME };
const SHIFT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTSHIFT as u16, value: 2, time: DUMMY_TIME };

const LEFTALT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 0, time: DUMMY_TIME };
const LEFTALT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 1, time: DUMMY_TIME };
const LEFTALT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 2, time: DUMMY_TIME };

const RIGHTALT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHTALT as u16, value: 0, time: DUMMY_TIME };
const RIGHTALT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHTALT as u16, value: 1, time: DUMMY_TIME };
const RIGHTALT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHTALT as u16, value: 2, time: DUMMY_TIME };

const CAPSLOCK_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_CAPSLOCK as u16, value: 0, time: DUMMY_TIME };
const CAPSLOCK_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_CAPSLOCK as u16, value: 1, time: DUMMY_TIME };
const CAPSLOCK_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_CAPSLOCK as u16, value: 2, time: DUMMY_TIME };

const LEFTCTRL_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTCTRL as u16, value: 0, time: DUMMY_TIME };
const LEFTCTRL_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTCTRL as u16, value: 1, time: DUMMY_TIME };
const LEFTCTRL_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTCTRL as u16, value: 2, time: DUMMY_TIME };

const ESC_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_ESC as u16, value: 0, time: DUMMY_TIME };
const ESC_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_ESC as u16, value: 1, time: DUMMY_TIME };
const ESC_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_ESC as u16, value: 2, time: DUMMY_TIME };

const H_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_H as u16, value: 0, time: DUMMY_TIME };
const H_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_H as u16, value: 1, time: DUMMY_TIME };
const H_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_H as u16, value: 2, time: DUMMY_TIME };
const J_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_J as u16, value: 0, time: DUMMY_TIME };
const J_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_J as u16, value: 1, time: DUMMY_TIME };
const J_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_J as u16, value: 2, time: DUMMY_TIME };
const K_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_K as u16, value: 0, time: DUMMY_TIME };
const K_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_K as u16, value: 1, time: DUMMY_TIME };
const K_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_K as u16, value: 2, time: DUMMY_TIME };
const L_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_L as u16, value: 0, time: DUMMY_TIME };
const L_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_L as u16, value: 1, time: DUMMY_TIME };
const L_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_L as u16, value: 2, time: DUMMY_TIME };

const ARROW_LEFT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFT as u16, value: 0, time: DUMMY_TIME };
const ARROW_LEFT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFT as u16, value: 1, time: DUMMY_TIME };
const ARROW_LEFT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFT as u16, value: 2, time: DUMMY_TIME };
const ARROW_DOWN_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_DOWN as u16, value: 0, time: DUMMY_TIME };
const ARROW_DOWN_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_DOWN as u16, value: 1, time: DUMMY_TIME };
const ARROW_DOWN_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_DOWN as u16, value: 2, time: DUMMY_TIME };
const ARROW_RIGHT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHT as u16, value: 0, time: DUMMY_TIME };
const ARROW_RIGHT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHT as u16, value: 1, time: DUMMY_TIME };
const ARROW_RIGHT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHT as u16, value: 2, time: DUMMY_TIME };
const ARROW_UP_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_UP as u16, value: 0, time: DUMMY_TIME };
const ARROW_UP_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_UP as u16, value: 1, time: DUMMY_TIME };
const ARROW_UP_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_UP as u16, value: 2, time: DUMMY_TIME };

const F8_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_F8 as u16, value: 0, time: DUMMY_TIME };
const F8_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_F8 as u16, value: 1, time: DUMMY_TIME };
const F8_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_F8 as u16, value: 2, time: DUMMY_TIME };

const MOUSE5_UP: input_event = input_event { type_: EV_KEY as u16, code: 277 as u16, value: 0, time: DUMMY_TIME };
const MOUSE5_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 277 as u16, value: 1, time: DUMMY_TIME };
const MOUSE5_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 277 as u16, value: 2, time: DUMMY_TIME };
const MOUSE6_UP: input_event = input_event { type_: EV_KEY as u16, code: 278 as u16, value: 0, time: DUMMY_TIME };
const MOUSE6_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 278 as u16, value: 1, time: DUMMY_TIME };
const MOUSE6_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 278 as u16, value: 2, time: DUMMY_TIME };
const MOUSE7_UP: input_event = input_event { type_: EV_KEY as u16, code: 279 as u16, value: 0, time: DUMMY_TIME };
const MOUSE7_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 279 as u16, value: 1, time: DUMMY_TIME };
const MOUSE7_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 279 as u16, value: 2, time: DUMMY_TIME };
const MOUSE8_UP: input_event = input_event { type_: EV_KEY as u16, code: 280 as u16, value: 0, time: DUMMY_TIME };
const MOUSE8_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 280 as u16, value: 1, time: DUMMY_TIME };
const MOUSE8_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 280 as u16, value: 2, time: DUMMY_TIME };
const MOUSE9_UP: input_event = input_event { type_: EV_KEY as u16, code: 281 as u16, value: 0, time: DUMMY_TIME };
const MOUSE9_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 281 as u16, value: 1, time: DUMMY_TIME };
const MOUSE9_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 281 as u16, value: 2, time: DUMMY_TIME };
const MOUSE10_UP: input_event = input_event { type_: EV_KEY as u16, code: 282 as u16, value: 0, time: DUMMY_TIME };
const MOUSE10_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 282 as u16, value: 1, time: DUMMY_TIME };
const MOUSE10_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 282 as u16, value: 2, time: DUMMY_TIME };
const MOUSE11_UP: input_event = input_event { type_: EV_KEY as u16, code: 283 as u16, value: 0, time: DUMMY_TIME };
const MOUSE11_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 283 as u16, value: 1, time: DUMMY_TIME };
const MOUSE11_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 283 as u16, value: 2, time: DUMMY_TIME };
const MOUSE12_UP: input_event = input_event { type_: EV_KEY as u16, code: 284 as u16, value: 0, time: DUMMY_TIME };
const MOUSE12_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 284 as u16, value: 1, time: DUMMY_TIME };
const MOUSE12_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 284 as u16, value: 2, time: DUMMY_TIME };

const WHEEL: input_event = input_event { type_: EV_REL as u16, code: REL_WHEEL as u16, value: 0, time: DUMMY_TIME };

const SYN: input_event = input_event { type_: EV_SYN as u16, code: SYN_REPORT as u16, value: 0, time: DUMMY_TIME };
// endregion

// region key references
static MOUSE5: Key = make_key(277);
static MOUSE6: Key = make_key(278);
static MOUSE7: Key = make_key(279);
static MOUSE8: Key = make_key(280);
static MOUSE9: Key = make_key(281);
static MOUSE10: Key = make_key(282);
static MOUSE11: Key = make_key(283);
static MOUSE12: Key = make_key(284);
static LEFT_META: Key = make_key(KEY_LEFTMETA);
static LEFT_ALT: Key = make_key(KEY_LEFTALT);
static RIGHT_ALT: Key = make_key(KEY_RIGHTALT);
static LEFT_SHIFT: Key = make_key(KEY_LEFTSHIFT);
static LEFT_CTRL: Key = make_key(KEY_LEFTCTRL);
static TAB: Key = make_key(KEY_TAB);
static CAPSLOCK: Key = make_key(KEY_CAPSLOCK);
static T: Key = make_key(KEY_T);
static W: Key = make_key(KEY_W);
static H: Key = make_key(KEY_H);
static J: Key = make_key(KEY_J);
static K: Key = make_key(KEY_K);
static L: Key = make_key(KEY_L);
static LEFT: Key = make_key(KEY_LEFT);
static RIGHT: Key = make_key(KEY_RIGHT);
static UP: Key = make_key(KEY_UP);
static DOWN: Key = make_key(KEY_DOWN);
static F5: Key = make_key(KEY_F5);
static KPD0: Key = make_key(KEY_KP0);
static KPD1: Key = make_key(KEY_KP1);
static KPD2: Key = make_key(KEY_KP2);
static KPD3: Key = make_key(KEY_KP3);
static KPD4: Key = make_key(KEY_KP4);
static KPD5: Key = make_key(KEY_KP5);
static KPD6: Key = make_key(KEY_KP6);
static KPD7: Key = make_key(KEY_KP7);

static TYPE_UP: i32 = 0;
static TYPE_DOWN: i32 = 1;
static TYPE_REPEAT: i32 = 2;
// endregion


fn is_modifier_down(state: &State) -> bool {
    return state.leftalt_is_down || state.leftcontrol_is_down || state.shift_is_down || state.meta_is_down;
}

fn ev_ignored(ev: &input_event, ignore_list: &mut Vec<input_event>) -> bool {
    match ignore_list.iter().position(|x: &input_event| x == ev) {
        None => false,
        Some(_) => true
    }
}

fn unignore_ev(ev: &input_event, ignore_list: &mut Vec<input_event>) {
    if let Some(pos) = ignore_list.iter().position(|x: &input_event| x == ev) {
        ignore_list.remove(pos);
    }
}

fn ignore_ev(ev: &input_event, ignore_list: &mut Vec<input_event>) {
    if let None = ignore_list.iter().position(|x: &input_event| equal(x, ev)) {
        ignore_list.push(*ev);
    }
}

fn log_msg(msg: &str) {
    let out_msg = format!("[DEBUG] {}\n", msg);

    io::stderr().write_all(out_msg.as_bytes()).unwrap();
}


enum ScopeInstruction {
    Scope(Scope),
    KeyMapping(KeyMappings),
}

struct Scope {
    condition: Option<KeyActionCondition>,
    instructions: Vec<ScopeInstruction>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let mut stdin = tokio::io::stdin();
    let mut read_ev: input_event = unsafe { mem::zeroed() };

    let (window_ev_tx, mut window_ev_rx) = tokio::sync::mpsc::channel(128);
    let (input_ev_tx, mut input_ev_rx) = tokio::sync::mpsc::channel(128);
    let (delay_tx, mut delay_rx) = tokio::sync::mpsc::channel(128);

    // x11 thread
    tokio::spawn(async move {
        let x11_state = Arc::new(x11_initialize().unwrap());

        loop {
            let x11_state_clone = x11_state.clone();
            let res = task::spawn_blocking(move || {
                x11_test(&x11_state_clone)
            }).await.unwrap();

            if let Ok(Some(val)) = res {
                window_ev_tx.send(val).await.unwrap_or_else(|_| panic!());
            }
        }
    });

    // input ev thread
    tokio::spawn(async move {
        loop {
            listen_to_key_events(&mut read_ev, &mut stdin).await;
            input_ev_tx.send(read_ev).await.unwrap();
        }
    });


    let mut state = State {
        tab_is_down: false,
        capslock_is_down: false,
        leftcontrol_is_down: false,
        shift_is_down: false,
        meta_is_down: false,
        leftalt_is_down: false,
        right_alt_is_down: false,
        disable_alt_mod: false,
        ignore_list: IgnoreList::new(),
        mappings: KeyMappings::new(),
        active_window: None,
    };

    let mut global_scope = Scope {
        condition: None,
        instructions: vec![],
    };

    let mut global_mappings = KeyMappings::new();

    global_mappings.replace_key_click(KeyClickAction::new(MOUSE5), KeyClickAction::new(KPD0));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE6), KeyClickAction::new(KPD1));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE7), KeyClickAction::new(KPD2));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE8), KeyClickAction::new(KPD3));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE9), KeyClickAction::new(KPD4));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE10), KeyClickAction::new(KPD5));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE11), KeyClickAction::new(KPD6));
    global_mappings.replace_key_click(KeyClickAction::new(MOUSE12), KeyClickAction::new(KPD7));


    { // figma-linux
        let mut local_mappings = KeyMappings::new();

        local_mappings.replace_key(KeyActionMods { key: MOUSE5, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(KeyActionMods { key: MOUSE5, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(
            KeyActionMods { key: MOUSE5, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
            KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}palette-pick{enter}".to_string()),
        );

        local_mappings.replace_key(KeyActionMods { key: MOUSE6, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(KeyActionMods { key: MOUSE6, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(
            KeyActionMods { key: MOUSE6, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
            KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}atom-sync{enter}".to_string()),
        );

        local_mappings.replace_key(KeyActionMods { key: MOUSE7, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(KeyActionMods { key: MOUSE7, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(
            KeyActionMods { key: MOUSE7, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
            KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}batch styler{enter}".to_string()),
        );

        local_mappings.replace_key(KeyActionMods { key: MOUSE8, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(KeyActionMods { key: MOUSE8, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(
            KeyActionMods { key: MOUSE8, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
            KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}chroma colors{enter}".to_string()),
        );

        local_mappings.replace_key(KeyActionMods { key: MOUSE9, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(KeyActionMods { key: MOUSE9, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(
            KeyActionMods { key: MOUSE9, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
            KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}scripter{enter}".to_string()),
        );

        local_mappings.replace_key(KeyActionMods { key: MOUSE11, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(KeyActionMods { key: MOUSE11, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
        local_mappings.replace_key(
            KeyActionMods { key: MOUSE11, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
            KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}theme-flip{enter}".to_string()),
        );

        global_scope.instructions.push(ScopeInstruction::Scope(Scope {
            condition: Some(KeyActionCondition { window_class_name: Some("figma-linux".to_string()) }),
            instructions: vec![ScopeInstruction::KeyMapping(local_mappings)],
        }));
    }

    { // arrow keys
        global_mappings.replace_key_click(
            KeyClickAction::new_mods(H, *KeyModifierFlags::new().alt()),
            KeyClickAction::new(LEFT),
        );
        global_mappings.replace_key_click(
            KeyClickAction::new_mods(L, *KeyModifierFlags::new().alt()),
            KeyClickAction::new(RIGHT),
        );
        global_mappings.replace_key_click(
            KeyClickAction::new_mods(K, *KeyModifierFlags::new().alt()),
            KeyClickAction::new(UP),
        );
        global_mappings.replace_key_click(
            KeyClickAction::new_mods(J, *KeyModifierFlags::new().alt()),
            KeyClickAction::new(DOWN),
        );
    }

    global_scope.instructions.push(ScopeInstruction::KeyMapping(global_mappings));


    { // firefox
        let mut local_mappings = KeyMappings::new();

        local_mappings.replace_key_click(KeyClickAction::new(MOUSE5), KeyClickAction::new_mods(TAB, *KeyModifierFlags::new().ctrl()));
        local_mappings.replace_key_click(KeyClickAction::new_mods(MOUSE5, *KeyModifierFlags::new().shift()), KeyClickAction::new_mods(TAB, *KeyModifierFlags::new().ctrl().shift()));
        local_mappings.replace_key_click(KeyClickAction::new(MOUSE6), KeyClickAction::new_mods(T, *KeyModifierFlags::new().ctrl()));
        local_mappings.replace_key_click(KeyClickAction::new(MOUSE7), KeyClickAction::new(F5));
        local_mappings.replace_key_click(KeyClickAction::new(MOUSE12), KeyClickAction::new_mods(W, *KeyModifierFlags::new().ctrl()));

        global_scope.instructions.push(ScopeInstruction::Scope(Scope {
            condition: Some(KeyActionCondition { window_class_name: Some("firefox".to_string()) }),
            instructions: vec![ScopeInstruction::KeyMapping(local_mappings)],
        }));
    }

    type Cache = HashMap<String, KeyMappings>;
    let mut cache: Cache = HashMap::new();

    fn eval_scope(scope: &Scope, state: &mut State, cache: &mut Cache) {
        if let Some(active_window) = &state.active_window {
            if let Some(cached) = cache.get(&active_window.class) {
                state.mappings.0.extend(
                    cached.0.iter().map(|(k, v)| { (k.clone(), v.clone()) })
                );
                return;
            }
        }

        // check condition
        if let Some(cond) = &scope.condition {
            if let Some(window_class_name) = &cond.window_class_name {
                if let Some(active_window) = &state.active_window {
                    if *window_class_name != active_window.class {
                        return;
                    }
                } else {
                    return;
                }
            }
        }

        for instruction in &scope.instructions {
            match instruction {
                ScopeInstruction::Scope(sub_scope) => { eval_scope(sub_scope, state, cache); }
                ScopeInstruction::KeyMapping(mapping) => {
                    state.mappings.0.extend(
                        mapping.0.iter().map(|(k, v)| { (k.clone(), v.clone()) })
                    );
                }
            }
        }

        // cache for later
        if let Some(active_window) = &state.active_window {
            cache.insert(active_window.class.to_string(), state.mappings.clone());
        }
    }

    eval_scope(&global_scope, &mut state, &mut cache);

    fn handle_active_window_change(scope: &Scope, state: &mut State, cache: &mut Cache) {
        state.mappings = KeyMappings::new();

        eval_scope(scope, state, cache);
    }

    loop {
        tokio::select! {
            Some(window) = window_ev_rx.recv() => {
                state.active_window = Some(window);
                handle_active_window_change(&global_scope, &mut state, &mut cache);
            }
            Some(ev) = input_ev_rx.recv() => {
                handle_stdin_ev(&mut state, &ev, delay_tx.clone()).unwrap();
            }
            Some(seq) = delay_rx.recv() => {
                process_key_sequence(&mut state.ignore_list, &seq, delay_tx.clone()).unwrap();
            }
            else => { break }
        }
    }

    Ok(())
}

fn make_event(type_: u16, code: u16, value: i32) -> input_event {
    input_event { type_, code, value, time: DUMMY_TIME }
}

async fn listen_to_key_events(ev: &mut input_event, input: &mut tokio::io::Stdin) {
    unsafe {
        let slice = any_as_u8_slice_mut(ev);
        match input.read_exact(slice).await {
            Ok(_) => (),
            Err(_) => {
                ::std::mem::forget(slice);
                panic!("error reading stdin");
            }
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Key { key_type: i32, code: i32 }

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum KeyModifierStateType {
    KEEP,
    UP,
    DOWN,
}

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
struct KeyModifierFlags {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
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
struct KeyModifierAction {
    ctrl: KeyModifierStateType,
    shift: KeyModifierStateType,
    alt: KeyModifierStateType,
    meta: KeyModifierStateType,
}

enum KeyModifierWithState {
    CTRL(KeyModifierStateType),
    SHIFT(KeyModifierStateType),
    ALT(KeyModifierStateType),
    META(KeyModifierStateType),
}

#[derive(Clone, Hash)]
enum KeySequenceItem {
    KeyAction(KeyAction),
    EatKeyAction(KeyAction),
    SleepAction(time::Duration),
}

#[derive(Clone, Hash)]
struct KeySequence(Vec<KeySequenceItem>);

type KEYCODE = i32;

trait KeycodeExt {
    fn to_key(&self) -> Key;
}

impl KeycodeExt for KEYCODE {
    fn to_key(&self) -> Key { Key { key_type: EV_KEY, code: *self } }
}

lazy_static! {
    static ref KEY_LOOKUP: HashMap<&'static str, KeySequence> = {
        let mut m = HashMap::new();
        m.insert("enter", KeySequence::new().append_click(KEY_ENTER.to_key()));
        m.insert("esc", KeySequence::new().append_click(KEY_ESC.to_key()));
        m.insert("ctrl", KeySequence::new().append_click(KEY_LEFTCTRL.to_key()));
        m.insert("ctrl down", KeySequence::new() .append_action(KeyAction::new(KEY_LEFTCTRL.to_key(), TYPE_DOWN)));
        m.insert("ctrl up", KeySequence::new() .append_action(KeyAction::new(KEY_LEFTCTRL.to_key(), TYPE_UP)));
        m.insert("shift", KeySequence::new().append_click(KEY_LEFTSHIFT.to_key()));
        m.insert(" ", KeySequence::new().append_click(KEY_SPACE.to_key()));
        m.insert("-", KeySequence::new().append_click(KEY_MINUS.to_key()));
        m.insert("/", KeySequence::new().append_click(KEY_SLASH.to_key()));
        m.insert("a", KeySequence::new().append_click(KEY_A.to_key()));
        m.insert("b", KeySequence::new().append_click(KEY_B.to_key()));
        m.insert("c", KeySequence::new().append_click(KEY_C.to_key()));
        m.insert("d", KeySequence::new().append_click(KEY_D.to_key()));
        m.insert("e", KeySequence::new().append_click(KEY_E.to_key()));
        m.insert("f", KeySequence::new().append_click(KEY_F.to_key()));
        m.insert("g", KeySequence::new().append_click(KEY_G.to_key()));
        m.insert("h", KeySequence::new().append_click(KEY_H.to_key()));
        m.insert("i", KeySequence::new().append_click(KEY_I.to_key()));
        m.insert("j", KeySequence::new().append_click(KEY_J.to_key()));
        m.insert("k", KeySequence::new().append_click(KEY_K.to_key()));
        m.insert("l", KeySequence::new().append_click(KEY_L.to_key()));
        m.insert("m", KeySequence::new().append_click(KEY_M.to_key()));
        m.insert("n", KeySequence::new().append_click(KEY_N.to_key()));
        m.insert("o", KeySequence::new().append_click(KEY_O.to_key()));
        m.insert("p", KeySequence::new().append_click(KEY_P.to_key()));
        m.insert("q", KeySequence::new().append_click(KEY_Q.to_key()));
        m.insert("r", KeySequence::new().append_click(KEY_R.to_key()));
        m.insert("s", KeySequence::new().append_click(KEY_S.to_key()));
        m.insert("t", KeySequence::new().append_click(KEY_T.to_key()));
        m.insert("u", KeySequence::new().append_click(KEY_U.to_key()));
        m.insert("v", KeySequence::new().append_click(KEY_V.to_key()));
        m.insert("w", KeySequence::new().append_click(KEY_W.to_key()));
        m.insert("x", KeySequence::new().append_click(KEY_X.to_key()));
        m.insert("y", KeySequence::new().append_click(KEY_Y.to_key()));
        m.insert("z", KeySequence::new().append_click(KEY_Z.to_key()));
        m.insert("V", KeySequence::new()
            .append_action(KeyAction::new(KEY_LEFTSHIFT.to_key(), TYPE_DOWN))
            .append_click(KEY_V.to_key())
            .append_action(KeyAction::new(KEY_LEFTSHIFT.to_key(), TYPE_UP))
        );
        m
    };
}

impl KeySequence {
    pub fn new() -> Self { KeySequence(Default::default()) }
    pub fn append(mut self, item: KeySequenceItem) -> Self {
        self.0.push(item);
        self
    }
    pub fn append_action(mut self, action: KeyAction) -> Self {
        self = self.append(KeySequenceItem::KeyAction(action));
        self
    }
    pub fn append_click(mut self, key: Key) -> Self {
        self = self.append_action(KeyAction::new(key, TYPE_DOWN));
        // self = self.sleep_for_millis(10);
        self = self.append_action(KeyAction::new(key, TYPE_UP));
        // self = self.sleep_for_millis(100);
        self
    }
    pub fn sleep_for_millis(mut self, duration: u64) -> Self {
        self.append(KeySequenceItem::SleepAction(time::Duration::from_millis(duration)))
    }

    pub fn append_string_sequence(mut self, sequence: String) -> Self {
        let mut it = sequence.chars();
        while let Some(ch) = it.next() {
            // special
            if ch == '{' {
                let special_char = it.by_ref().take_while(|&ch| ch != '}').collect::<String>();
                let seq = KEY_LOOKUP.get(special_char.as_str())
                    .expect(format!("failed to lookup key '{}'", special_char).as_str());
                self.0.extend(seq.0.iter().cloned());
                continue;
            }

            let seq = KEY_LOOKUP.get(ch.to_string().as_str())
                .expect(format!("failed to lookup key '{}'", ch).as_str());
            self.0.extend(seq.0.iter().cloned());
        }

        self
    }
}

impl KeyModifierAction {
    fn new() -> KeyModifierAction {
        use KeyModifierStateType::*;
        KeyModifierAction { ctrl: KEEP, shift: KEEP, alt: KEEP, meta: KEEP }
    }
    fn apply(mut self, modifier: KeyModifierWithState) -> Self {
        match modifier {
            KeyModifierWithState::CTRL(state) => { self.ctrl = state }
            KeyModifierWithState::SHIFT(state) => { self.shift = state }
            KeyModifierWithState::ALT(state) => { self.alt = state }
            KeyModifierWithState::META(state) => { self.meta = state }
        };
        self
    }
    fn inverse(&self) -> KeyModifierAction {
        use KeyModifierStateType::*;
        fn invert_state(state: &KeyModifierStateType) -> KeyModifierStateType {
            if *state == DOWN { return UP; }
            if *state == UP { return DOWN; }
            return KEEP;
        }

        let mut inverted = Self::new();
        inverted.ctrl = invert_state(&self.ctrl);
        inverted.alt = invert_state(&self.alt);
        inverted.meta = invert_state(&self.meta);
        inverted.shift = invert_state(&self.shift);
        inverted
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyAction { key: Key, value: i32 }

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyActionMods { key: Key, value: i32, modifiers: KeyModifierFlags }

impl KeyAction { pub fn new(key: Key, value: i32) -> Self { KeyAction { key, value } } }

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct KeyClickAction { key: Key, modifiers: KeyModifierFlags }

impl KeyClickAction {
    pub fn new(key: Key) -> Self { KeyClickAction { key, modifiers: KeyModifierFlags::new() } }
    pub fn new_mods(key: Key, modifiers: KeyModifierFlags) -> Self { KeyClickAction { key, modifiers } }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct KeyActionCondition { window_class_name: Option<String> }

#[derive(Clone)]
struct KeyMappings(HashMap<KeyActionMods, KeySequence>);

impl KeyMappings {
    fn new() -> Self {
        KeyMappings { 0: Default::default() }
    }

    fn replace_key(&mut self, from: KeyActionMods, to: KeySequence) {
        self.0.insert(from, to);
    }

    fn replace_key_click(&mut self, from: KeyClickAction, to: KeyClickAction) {
        {
            let mut to_seq = KeySequence::new();

            if from.modifiers.ctrl && !to.modifiers.ctrl {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_CTRL, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(LEFT_CTRL, TYPE_UP)));
            }
            if from.modifiers.alt && !to.modifiers.alt {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_ALT, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(LEFT_ALT, TYPE_UP)));
            }
            if from.modifiers.shift && !to.modifiers.shift {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_SHIFT, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(LEFT_SHIFT, TYPE_UP)));
            }
            if from.modifiers.meta && !to.modifiers.meta {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_META, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(LEFT_META, TYPE_UP)));
            }

            if to.modifiers.ctrl && !from.modifiers.ctrl { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_CTRL, value: TYPE_DOWN })) }
            if to.modifiers.alt && !from.modifiers.alt { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_ALT, value: TYPE_DOWN })) }
            if to.modifiers.shift && !from.modifiers.shift { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_SHIFT, value: TYPE_DOWN })) }
            if to.modifiers.meta && !from.modifiers.meta { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_META, value: TYPE_DOWN })) }

            to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));

            self.replace_key(
                KeyActionMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() },
                to_seq,
            );
        }

        {
            let mut to_seq = KeySequence::new();
            to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

            if to.modifiers.ctrl && !from.modifiers.ctrl { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_CTRL, value: TYPE_UP })) }
            if to.modifiers.alt && !from.modifiers.alt { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_ALT, value: TYPE_UP })) }
            if to.modifiers.shift && !from.modifiers.shift { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_SHIFT, value: TYPE_UP })) }
            if to.modifiers.meta && !from.modifiers.meta { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: LEFT_META, value: TYPE_UP })) }

            self.replace_key(
                KeyActionMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() },
                to_seq,
            );
        }

        {
            let mut to_seq = KeySequence::new();
            to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: to.key, value: TYPE_REPEAT }));

            self.replace_key(
                KeyActionMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers },
                to_seq,
            );
        }
    }
}

const fn make_key(code: i32) -> Key { Key { key_type: EV_KEY, code } }


fn update_modifiers(state: &mut State, ev: &input_event) {
    if *ev == LEFTCTRL_DOWN {
        state.leftcontrol_is_down = true;
    } else if *ev == LEFTCTRL_UP {
        state.leftcontrol_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(LEFT_CTRL, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(LEFT_CTRL, TYPE_UP));
            return;
        }
    }

    if *ev == LEFTALT_DOWN {
        state.leftalt_is_down = true;
    } else if *ev == LEFTALT_UP {
        state.leftalt_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(LEFT_ALT, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(LEFT_ALT, TYPE_UP));
            return;
        }
    }

    if *ev == SHIFT_DOWN {
        state.shift_is_down = true;
    } else if *ev == SHIFT_UP {
        state.shift_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(LEFT_SHIFT, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(LEFT_SHIFT, TYPE_UP));
            return;
        }
    }

    if *ev == META_DOWN {
        state.meta_is_down = true;
    } else if *ev == META_UP {
        state.meta_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(LEFT_META, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(LEFT_META, TYPE_UP));
            return;
        }
    }
}

fn handle_stdin_ev(mut state: &mut State, ev: &input_event, delay_tx: tokio::sync::mpsc::Sender<KeySequence>) -> Result<()> {
    if ev.type_ != EV_KEY as u16 {
        print_event(&ev);
        return Ok(());
    }

    update_modifiers(&mut state, &ev);

    if crate::tab_mod::tab_mod(&ev, &mut *state) {
        return Ok(());
    }

    if !state.leftcontrol_is_down {
        if crate::caps_mod::caps_mod(&ev, &mut *state) {
            return Ok(());
        }
    }

    if !state.disable_alt_mod {
        if crate::rightalt_mod::rightalt_mod(&ev, &mut *state) {
            return Ok(());
        }
    }

    let mut from_modifiers = KeyModifierFlags::new();
    from_modifiers.ctrl = state.leftcontrol_is_down.clone();
    from_modifiers.alt = state.leftalt_is_down.clone();
    from_modifiers.shift = state.shift_is_down.clone();
    from_modifiers.meta = state.meta_is_down.clone();

    let from_key_action = KeyActionMods {
        key: Key { key_type: ev.type_ as i32, code: ev.code as i32 },
        value: ev.value,
        modifiers: from_modifiers,
    };

    if let Some(to_action_seq) = state.mappings.0.get(&from_key_action) {
        process_key_sequence(&mut state.ignore_list, to_action_seq, delay_tx);
        return Ok(());
    }

    print_event(&ev);

    Ok(())
}

fn process_key_sequence(ignore_list: &mut IgnoreList, to_action_seq: &KeySequence, delay_tx: Sender<KeySequence>) -> Result<()> {
    for (idx, seq_item) in to_action_seq.0.iter().enumerate() {
        match seq_item {
            KeySequenceItem::KeyAction(action) => {
                print_event(&make_event(
                    action.key.key_type as u16,
                    action.key.code as u16,
                    action.value));

                print_event(&SYN);
                thread::sleep(time::Duration::from_micros(20000));
            }
            KeySequenceItem::EatKeyAction(key_action) => {
                ignore_list.ignore(&key_action);
            }
            KeySequenceItem::SleepAction(duration) => {
                let duration = duration.clone();
                let seq = KeySequence(to_action_seq.0.iter().skip(idx + 1).map(|v| v.clone()).collect());
                tokio::spawn(async move {
                    tokio::time::sleep(duration).await;
                    delay_tx.send(seq).await;
                });
                return Ok(());
            }
        }
    }
    return Ok(());
}
