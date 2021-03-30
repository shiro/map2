#![feature(type_ascription)]
#![feature(async_closure)]
#![feature(impl_trait_in_bindings)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(label_break_value)]

use tokio::prelude::*;
use futures::future::{Future, lazy, select};

mod tab_mod;
mod caps_mod;
mod leftalt_mod;
mod rightalt_mod;
mod x11;

use tokio::prelude::*;
use std::process::exit;
use std::{io, mem, slice, thread, time};
use std::io::{stdout, Write};

use input_linux_sys::{KEY_E, KEY_K, KEY_J, EV_KEY, KEY_TAB, KEY_LEFTMETA, KEY_LEFTSHIFT, KEY_LEFTALT, EV_SYN, SYN_REPORT, EV_MSC, MSC_SCAN, KEY_CAPSLOCK, KEY_LEFTCTRL, KEY_ESC, KEY_H, KEY_L, KEY_LEFT, KEY_DOWN, KEY_RIGHT, KEY_UP, KEY_RIGHTALT, KEY_F8, REL_Y, REL_X, EV_REL, KEY_F13, KEY_A, KEY_F14, KEY_F15, KEY_F16, KEY_F17, KEY_NUMERIC_0, KEY_NUMERIC_1, KEY_NUMERIC_3, KEY_NUMERIC_4, KEY_NUMERIC_5, KEY_NUMERIC_6, KEY_NUMERIC_7, KEY_NUMERIC_8, KEY_KP0, KEY_KP1, KEY_KP2, KEY_KP3, KEY_KP4, KEY_KP5, KEY_KP6, KEY_KP7};
use crate::x11::{x11_get_active_window, x11_test, x11_initialize};
use tokio::task;
use anyhow::Result;
use tokio::time::Duration;
use std::sync::Arc;
use nom::lib::std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use crate::x11::ActiveWindowResult;

pub type time_t = i64;
pub type suseconds_t = i64;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
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

pub struct State {
    tab_is_down: bool,
    capslock_is_down: bool,
    leftcontrol_is_down: bool,
    shift_is_down: bool,
    meta_is_down: bool,
    leftalt_is_down: bool,
    right_alt_is_down: bool,

    disable_alt_mod: bool,

    ignore_list: Vec<input_event>,
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

static EV_SIZE: usize = mem::size_of::<input_event>();

fn print_event(ev: &input_event) {
    unsafe {
        stdout().write_all(any_as_u8_slice(ev)).unwrap();
        // if ev.type_ == EV_KEY as u16 {
        //     println!("{:?}", ev);
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
static DUMMY_TIME: timeval = timeval { tv_sec: 0, tv_usec: 0 };

static TAB_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_TAB as u16, value: 0, time: DUMMY_TIME };
static TAB_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_TAB as u16, value: 1, time: DUMMY_TIME };
static TAB_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_TAB as u16, value: 2, time: DUMMY_TIME };

static META_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTMETA as u16, value: 0, time: DUMMY_TIME };
static META_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTMETA as u16, value: 1, time: DUMMY_TIME };
static META_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTMETA as u16, value: 2, time: DUMMY_TIME };

static SHIFT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTSHIFT as u16, value: 0, time: DUMMY_TIME };
static SHIFT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTSHIFT as u16, value: 1, time: DUMMY_TIME };
static SHIFT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTSHIFT as u16, value: 2, time: DUMMY_TIME };

static LEFTALT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 0, time: DUMMY_TIME };
static LEFTALT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 1, time: DUMMY_TIME };
static LEFTALT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 2, time: DUMMY_TIME };

static RIGHTALT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHTALT as u16, value: 0, time: DUMMY_TIME };
static RIGHTALT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHTALT as u16, value: 1, time: DUMMY_TIME };
static RIGHTALT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHTALT as u16, value: 2, time: DUMMY_TIME };

static CAPSLOCK_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_CAPSLOCK as u16, value: 0, time: DUMMY_TIME };
static CAPSLOCK_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_CAPSLOCK as u16, value: 1, time: DUMMY_TIME };
static CAPSLOCK_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_CAPSLOCK as u16, value: 2, time: DUMMY_TIME };

static LEFTCTRL_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTCTRL as u16, value: 0, time: DUMMY_TIME };
static LEFTCTRL_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTCTRL as u16, value: 1, time: DUMMY_TIME };
static LEFTCTRL_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTCTRL as u16, value: 2, time: DUMMY_TIME };

static ESC_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_ESC as u16, value: 0, time: DUMMY_TIME };
static ESC_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_ESC as u16, value: 1, time: DUMMY_TIME };
static ESC_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_ESC as u16, value: 2, time: DUMMY_TIME };

static H_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_H as u16, value: 0, time: DUMMY_TIME };
static H_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_H as u16, value: 1, time: DUMMY_TIME };
static H_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_H as u16, value: 2, time: DUMMY_TIME };
static J_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_J as u16, value: 0, time: DUMMY_TIME };
static J_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_J as u16, value: 1, time: DUMMY_TIME };
static J_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_J as u16, value: 2, time: DUMMY_TIME };
static K_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_K as u16, value: 0, time: DUMMY_TIME };
static K_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_K as u16, value: 1, time: DUMMY_TIME };
static K_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_K as u16, value: 2, time: DUMMY_TIME };
static L_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_L as u16, value: 0, time: DUMMY_TIME };
static L_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_L as u16, value: 1, time: DUMMY_TIME };
static L_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_L as u16, value: 2, time: DUMMY_TIME };

static ARROW_LEFT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFT as u16, value: 0, time: DUMMY_TIME };
static ARROW_LEFT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFT as u16, value: 1, time: DUMMY_TIME };
static ARROW_LEFT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFT as u16, value: 2, time: DUMMY_TIME };
static ARROW_DOWN_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_DOWN as u16, value: 0, time: DUMMY_TIME };
static ARROW_DOWN_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_DOWN as u16, value: 1, time: DUMMY_TIME };
static ARROW_DOWN_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_DOWN as u16, value: 2, time: DUMMY_TIME };
static ARROW_RIGHT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHT as u16, value: 0, time: DUMMY_TIME };
static ARROW_RIGHT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHT as u16, value: 1, time: DUMMY_TIME };
static ARROW_RIGHT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_RIGHT as u16, value: 2, time: DUMMY_TIME };
static ARROW_UP_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_UP as u16, value: 0, time: DUMMY_TIME };
static ARROW_UP_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_UP as u16, value: 1, time: DUMMY_TIME };
static ARROW_UP_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_UP as u16, value: 2, time: DUMMY_TIME };

static F8_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_F8 as u16, value: 0, time: DUMMY_TIME };
static F8_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_F8 as u16, value: 1, time: DUMMY_TIME };
static F8_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_F8 as u16, value: 2, time: DUMMY_TIME };

static MOUSE5_UP: input_event = input_event { type_: EV_KEY as u16, code: 277 as u16, value: 0, time: DUMMY_TIME };
static MOUSE5_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 277 as u16, value: 1, time: DUMMY_TIME };
static MOUSE5_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 277 as u16, value: 2, time: DUMMY_TIME };
static MOUSE6_UP: input_event = input_event { type_: EV_KEY as u16, code: 278 as u16, value: 0, time: DUMMY_TIME };
static MOUSE6_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 278 as u16, value: 1, time: DUMMY_TIME };
static MOUSE6_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 278 as u16, value: 2, time: DUMMY_TIME };
static MOUSE7_UP: input_event = input_event { type_: EV_KEY as u16, code: 279 as u16, value: 0, time: DUMMY_TIME };
static MOUSE7_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 279 as u16, value: 1, time: DUMMY_TIME };
static MOUSE7_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 279 as u16, value: 2, time: DUMMY_TIME };
static MOUSE8_UP: input_event = input_event { type_: EV_KEY as u16, code: 280 as u16, value: 0, time: DUMMY_TIME };
static MOUSE8_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 280 as u16, value: 1, time: DUMMY_TIME };
static MOUSE8_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 280 as u16, value: 2, time: DUMMY_TIME };
static MOUSE9_UP: input_event = input_event { type_: EV_KEY as u16, code: 281 as u16, value: 0, time: DUMMY_TIME };
static MOUSE9_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 281 as u16, value: 1, time: DUMMY_TIME };
static MOUSE9_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 281 as u16, value: 2, time: DUMMY_TIME };
static MOUSE10_UP: input_event = input_event { type_: EV_KEY as u16, code: 282 as u16, value: 0, time: DUMMY_TIME };
static MOUSE10_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 282 as u16, value: 1, time: DUMMY_TIME };
static MOUSE10_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 282 as u16, value: 2, time: DUMMY_TIME };
static MOUSE11_UP: input_event = input_event { type_: EV_KEY as u16, code: 283 as u16, value: 0, time: DUMMY_TIME };
static MOUSE11_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 283 as u16, value: 1, time: DUMMY_TIME };
static MOUSE11_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 283 as u16, value: 2, time: DUMMY_TIME };
static MOUSE12_UP: input_event = input_event { type_: EV_KEY as u16, code: 284 as u16, value: 0, time: DUMMY_TIME };
static MOUSE12_DOWN: input_event = input_event { type_: EV_KEY as u16, code: 284 as u16, value: 1, time: DUMMY_TIME };
static MOUSE12_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: 284 as u16, value: 2, time: DUMMY_TIME };

static SYN: input_event = input_event { type_: EV_SYN as u16, code: SYN_REPORT as u16, value: 0, time: DUMMY_TIME };
// endregion


fn is_modifier_down(state: &State) -> bool {
    return state.leftalt_is_down || state.leftcontrol_is_down || state.shift_is_down || state.meta_is_down;
}

fn ev_ignored(ev: &input_event, ignore_list: &mut Vec<input_event>) -> bool {
    match ignore_list.iter().position(|x: &input_event| equal(x, ev)) {
        None => false,
        Some(_) => true
    }
}

fn unignore_ev(ev: &input_event, ignore_list: &mut Vec<input_event>) {
    if let Some(pos) = ignore_list.iter().position(|x: &input_event| equal(x, ev)) {
        ignore_list.remove(pos);
    }
}

fn ignore_ev(ev: &input_event, ignore_list: &mut Vec<input_event>) {
    if let None = ignore_list.iter().position(|x: &input_event| equal(x, ev)) {
        ignore_list.push(*ev);
    }
}


async fn delay_for(seconds: u64) -> Result<u64, task::JoinError> {
    task::spawn_blocking(move || {
        thread::sleep(Duration::from_secs(seconds));
    })
        .await?;
    Ok(seconds)
}

async fn log_msg_async(msg: &str) {
    let out_msg = format!("[DEBUG] {}\n", msg);

    tokio::io::stderr().write_all(out_msg.as_bytes()).await.unwrap();
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
    log_msg_async("hi").await;

    let mut stdin = tokio::io::stdin();
    let mut read_ev: input_event = unsafe { mem::zeroed() };

    let (mut tx1, mut rx1) = tokio::sync::mpsc::channel(128);
    let (mut tx2, mut rx2) = tokio::sync::mpsc::channel(128);

    // x11 thread
    tokio::spawn(async move {
        let x11_state = Arc::new(x11_initialize().unwrap());

        loop {
            let x11_state_clone = x11_state.clone();
            let res = task::spawn_blocking(move || {
                x11_test(&x11_state_clone)
            }).await.unwrap();

            if let Ok(Some(val)) = res {
                tx1.send(val).await.unwrap_or_else(|_| panic!());
            }
        }
    });

    // input ev thread
    tokio::spawn(async move {
        loop {
            listen_to_key_events(&mut read_ev, &mut stdin).await;
            tx2.send(read_ev).await.unwrap();
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
        ignore_list: vec!(),
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

    global_scope.instructions.push(ScopeInstruction::KeyMapping(global_mappings));


    // TODO make the API prettier
    { // firefox
        let mut local_mappings = KeyMappings::new();

        let mut to = KeyClickAction::new(TAB);
        to.modifiers.ctrl = true;
        local_mappings.replace_key_click(KeyClickAction::new(MOUSE5), to);

        global_scope.instructions.push(ScopeInstruction::Scope(Scope {
            condition: Some(KeyActionCondition { window_class_name: Some("firefox".to_string()) }),
            instructions: vec![ScopeInstruction::KeyMapping(local_mappings)],
        }));
    }

    type Cache = HashMap<String, KeyMappings>;
    let mut cache: Cache = HashMap::new();

    fn eval_scope(scope: &Scope, state: &mut State, cache: &mut Cache) {
        log_msg("evaling scope");

        if let Some(active_window) = &state.active_window {
            if let Some(cached) = cache.get(&active_window.class) {
                state.mappings.0.extend(&cached.0);
                log_msg("cached");
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
                    state.mappings.0.extend(mapping.0.iter());
                }
            }
        }

        // cache for later
        if let Some(active_window) = &state.active_window {
            // cache.insert(active_window.class.to_string(), state.mappings.clone());
        }
    }

    eval_scope(&global_scope, &mut state, &mut cache);

    fn handle_active_window_change(scope: &Scope, state: &mut State, cache: &mut Cache) {
        state.mappings = KeyMappings::new();

        eval_scope(scope, state, cache);
    }

    loop {
        tokio::select! {
            Some(window) = rx1.recv() => {
                state.active_window = Some(window);
                handle_active_window_change(&global_scope,&mut state, &mut cache);
            }
            Some(ev) = rx2.recv() => {
                handle_stdin_ev(&mut state, &ev).unwrap();
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


#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Key { key_type: i32, code: i32 }

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct KeyModifiers {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

impl KeyModifiers {
    fn new() -> KeyModifiers {
        KeyModifiers { ctrl: false, shift: false, alt: false, meta: false }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct KeyAction { key: Key, value: i32, modifiers: KeyModifiers }

impl KeyAction { pub fn new(key: Key, value: i32) -> Self { KeyAction { key, value, modifiers: KeyModifiers::new() } } }

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct KeyClickAction { key: Key, modifiers: KeyModifiers }

impl KeyClickAction { pub fn new(key: Key) -> Self { KeyClickAction { key, modifiers: KeyModifiers::new() } } }

#[derive(Clone, Eq, PartialEq, Hash)]
struct KeyActionCondition { window_class_name: Option<String> }

#[derive(Clone, Eq, PartialEq)]
struct KeyMappings(HashMap<KeyAction, KeyAction>);

impl KeyMappings {
    fn new() -> Self {
        KeyMappings { 0: Default::default() }
    }

    fn replace_key(&mut self, from: KeyAction, to: KeyAction) {
        self.0.insert(from, to);
    }

    fn replace_key_click(&mut self, from: KeyClickAction, to: KeyClickAction) {
        self.replace_key(
            KeyAction { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers },
            KeyAction { key: to.key, value: TYPE_DOWN, modifiers: to.modifiers },
        );
        self.replace_key(
            KeyAction { key: from.key, value: TYPE_UP, modifiers: from.modifiers },
            KeyAction { key: to.key, value: TYPE_UP, modifiers: to.modifiers },
        );

        self.replace_key(
            KeyAction { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers },
            KeyAction { key: to.key, value: TYPE_REPEAT, modifiers: to.modifiers },
        );
    }
}

const fn make_key(code: i32) -> Key { Key { key_type: EV_KEY, code } }

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
static LEFT_SHIFT: Key = make_key(KEY_LEFTSHIFT);
static LEFT_CTRL: Key = make_key(KEY_LEFTCTRL);
static TAB: Key = make_key(KEY_TAB);
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

fn handle_stdin_ev(mut state: &mut State, ev: &input_event) -> Result<()> {
    if ev.type_ != EV_KEY as u16 {
        print_event(&ev);
        return Ok(());
    }

    if equal(&ev, &F8_DOWN) {
        state.disable_alt_mod = !state.disable_alt_mod;
    }

    if equal(&ev, &LEFTCTRL_DOWN) {
        state.leftcontrol_is_down = true;
    } else if equal(&ev, &LEFTCTRL_UP) {
        state.leftcontrol_is_down = false;
    }

    if equal(&ev, &SHIFT_DOWN) {
        state.shift_is_down = true;
    } else if equal(&ev, &SHIFT_UP) {
        state.shift_is_down = false;
    }

    if crate::tab_mod::tab_mod(&ev, state) {
        return Ok(());
    }

    if !state.leftcontrol_is_down {
        if crate::caps_mod::caps_mod(&ev, state) {
            return Ok(());
        }
    }

    if !state.disable_alt_mod {
        if crate::leftalt_mod::leftalt_mod(&ev, state) {
            return Ok(());
        }
    }

    if !state.disable_alt_mod {
        if crate::rightalt_mod::rightalt_mod(&ev, state) {
            return Ok(());
        }
    }

    let mappings = &mut state.mappings;
    let from_key_action = KeyAction {
        key: Key { key_type: ev.type_ as i32, code: ev.code as i32 },
        value: ev.value,
        modifiers: KeyModifiers {
            ctrl: state.leftcontrol_is_down.clone(),
            shift: state.shift_is_down.clone(),
            alt: state.leftalt_is_down.clone(),
            meta: state.meta_is_down.clone(),
        },
    };

    if let Some(to_action) = mappings.0.get(&from_key_action) {
        let mut using_modifiers = false;

        if to_action.key.key_type != EV_KEY || to_action.value != TYPE_REPEAT {
            if to_action.modifiers.meta {
                print_event(&make_event(
                    LEFT_META.key_type as u16,
                    LEFT_META.code as u16,
                    to_action.value));
                using_modifiers = true;
            }

            if to_action.modifiers.ctrl {
                print_event(&make_event(
                    LEFT_CTRL.key_type as u16,
                    LEFT_CTRL.code as u16,
                    to_action.value));
                using_modifiers = true;
            }

            if to_action.modifiers.alt {
                print_event(&make_event(
                    LEFT_ALT.key_type as u16,
                    LEFT_ALT.code as u16,
                    to_action.value));
                using_modifiers = true;
            }

            if to_action.modifiers.shift {
                print_event(&make_event(
                    LEFT_SHIFT.key_type as u16,
                    LEFT_SHIFT.code as u16,
                    to_action.value));
                using_modifiers = true;
            }
        }

        if using_modifiers {
            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
        }

        print_event(&make_event(
            to_action.key.key_type as u16,
            to_action.key.code as u16,
            to_action.value));

        return Ok(());
    }


    print_event(&ev);

    Ok(())
}
