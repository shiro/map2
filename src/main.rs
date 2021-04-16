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

use anyhow::{Result, Error};
use nom::lib::std::collections::HashMap;
use tokio::prelude::*;
use tokio::task;

use crate::x11::{x11_initialize, x11_test};
use crate::x11::ActiveWindowResult;

use crate::key_defs::*;
use crate::key_defs::input_event;

use crate::state::*;
use crate::scope::*;

mod tab_mod;
mod caps_mod;
mod rightalt_mod;
mod x11;
mod key_defs;
mod state;
mod scope;
mod mappings;

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


static TYPE_UP: i32 = 0;
static TYPE_DOWN: i32 = 1;
static TYPE_REPEAT: i32 = 2;


fn log_msg(msg: &str) {
    let out_msg = format!("[DEBUG] {}\n", msg);

    io::stderr().write_all(out_msg.as_bytes()).unwrap();
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

    let mut cache: ScopeCache = HashMap::new();
    let global_scope = mappings::bind_mappings(&mut state);

    eval_scope(&global_scope, &mut state, &mut cache);

    fn handle_active_window_change(scope: &Scope, state: &mut State, cache: &mut ScopeCache) {
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
enum KeyModifierStateType { KEEP, UP, DOWN }

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

#[derive(Clone, Hash, Debug)]
pub enum KeySequenceItem {
    KeyAction(KeyAction),
    EatKeyAction(KeyAction),
    SleepAction(time::Duration),
}

#[derive(Clone, Hash, Debug)]
pub struct KeySequence(Vec<KeySequenceItem>);


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
    pub fn sleep_for_millis(self, duration: u64) -> Self {
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

impl KeyAction {
    pub fn new(key: Key, value: i32) -> Self { KeyAction { key, value } }
    pub fn to_input_ev(&self) -> input_event {
        input_event { type_: self.key.key_type as u16, code: self.key.code as u16, value: self.value, time: INPUT_EV_DUMMY_TIME }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct KeyClickAction { key: Key, modifiers: KeyModifierFlags }

impl KeyClickAction {
    pub fn new(key: Key) -> Self { KeyClickAction { key, modifiers: KeyModifierFlags::new() } }
    pub fn new_mods(key: Key, modifiers: KeyModifierFlags) -> Self { KeyClickAction { key, modifiers } }
}

#[derive(Clone)]
pub struct KeyMappings(HashMap<KeyActionMods, KeySequence>);

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
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_CTRL, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(KEY_LEFT_CTRL, TYPE_UP)));
            }
            if from.modifiers.alt && !to.modifiers.alt {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_ALT, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(KEY_LEFT_ALT, TYPE_UP)));
            }
            if from.modifiers.shift && !to.modifiers.shift {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_SHIFT, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(KEY_LEFT_SHIFT, TYPE_UP)));
            }
            if from.modifiers.meta && !to.modifiers.meta {
                to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_META, value: TYPE_UP }));
                to_seq.0.push(KeySequenceItem::EatKeyAction(KeyAction::new(KEY_LEFT_META, TYPE_UP)));
            }

            if to.modifiers.ctrl && !from.modifiers.ctrl { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_CTRL, value: TYPE_DOWN })) }
            if to.modifiers.alt && !from.modifiers.alt { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_ALT, value: TYPE_DOWN })) }
            if to.modifiers.shift && !from.modifiers.shift { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_SHIFT, value: TYPE_DOWN })) }
            if to.modifiers.meta && !from.modifiers.meta { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_META, value: TYPE_DOWN })) }

            to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));

            self.replace_key(
                KeyActionMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() },
                to_seq,
            );
        }

        {
            let mut to_seq = KeySequence::new();
            to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

            if to.modifiers.ctrl && !from.modifiers.ctrl { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_CTRL, value: TYPE_UP })) }
            if to.modifiers.alt && !from.modifiers.alt { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_ALT, value: TYPE_UP })) }
            if to.modifiers.shift && !from.modifiers.shift { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_SHIFT, value: TYPE_UP })) }
            if to.modifiers.meta && !from.modifiers.meta { to_seq.0.push(KeySequenceItem::KeyAction(KeyAction { key: KEY_LEFT_META, value: TYPE_UP })) }

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


fn update_modifiers(state: &mut State, ev: &input_event) {
    if *ev == INPUT_EV_LEFTCTRL.down {
        state.leftcontrol_is_down = true;
    } else if *ev == INPUT_EV_LEFTCTRL.up {
        state.leftcontrol_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(KEY_LEFT_CTRL, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(KEY_LEFT_CTRL, TYPE_UP));
            return;
        }
    }

    if *ev == INPUT_EV_LEFTALT.down {
        state.leftalt_is_down = true;
    } else if *ev == INPUT_EV_LEFTALT.up {
        state.leftalt_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(KEY_LEFT_ALT, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(KEY_LEFT_ALT, TYPE_UP));
            return;
        }
    }

    if *ev == INPUT_EV_SHIFT.down {
        state.shift_is_down = true;
    } else if *ev == INPUT_EV_SHIFT.up {
        state.shift_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(KEY_LEFT_SHIFT, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(KEY_LEFT_SHIFT, TYPE_UP));
            return;
        }
    }

    if *ev == INPUT_EV_META.down {
        state.meta_is_down = true;
    } else if *ev == INPUT_EV_META.up {
        state.meta_is_down = false;

        if state.ignore_list.is_ignored(&KeyAction::new(KEY_LEFT_META, TYPE_UP)) {
            state.ignore_list.unignore(&KeyAction::new(KEY_LEFT_META, TYPE_UP));
            return;
        }
    }
}

fn handle_stdin_ev(mut state: &mut State, ev: &input_event, delay_tx: tokio::sync::mpsc::Sender<KeySequence>) -> Result<()> {
    if ev.type_ != input_linux_sys::EV_KEY as u16 {
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
        process_key_sequence(&mut state.ignore_list, to_action_seq, delay_tx)?;
        return Ok(());
    }

    print_event(&ev);

    Ok(())
}

fn process_key_sequence(ignore_list: &mut IgnoreList, to_action_seq: &KeySequence, delay_tx: tokio::sync::mpsc::Sender<KeySequence>) -> Result<()> {
    for (idx, seq_item) in to_action_seq.0.iter().enumerate() {
        match seq_item {
            KeySequenceItem::KeyAction(action) => {
                print_event(&action.to_input_ev());

                print_event(&INPUT_EV_SYN);
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
                    delay_tx.send(seq).await?;
                    Ok::<(), Error>(())
                });
                return Ok(());
            }
        }
    }
    return Ok(());
}
