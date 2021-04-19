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
use std::rc::Rc;
use std::sync::Mutex;
use futures::future::{BoxFuture, FutureExt};
use async_recursion::async_recursion;

use anyhow::{Result, Error};
use nom::lib::std::collections::HashMap;
use tokio::prelude::*;
use tokio::task;

use crate::x11::{x11_initialize, x11_test};
use crate::x11::ActiveWindowInfo;

use crate::key_primitives::*;
use crate::key_defs::*;
use crate::key_defs::input_event;

use crate::state::*;
use crate::scope::*;
use std::cell::RefCell;

use futures::executor;

mod tab_mod;
mod caps_mod;
mod rightalt_mod;
mod x11;
mod key_defs;
mod state;
mod scope;
mod mappings;
mod block_ext;
mod key_primitives;

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

fn log_msg(msg: &str) {
    let out_msg = format!("[DEBUG] {}\n", msg);

    io::stderr().write_all(out_msg.as_bytes()).unwrap();
}


pub(crate) enum ExecutionMessage {
    EatEv(KeyAction),
    AddMapping(usize, KeyActionWithMods, Block),
}

pub(crate) type ExecutionMessageSender = tokio::sync::mpsc::Sender<ExecutionMessage>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stdin = tokio::io::stdin();
    let mut read_ev: input_event = unsafe { mem::zeroed() };

    let (window_ev_tx, mut window_ev_rx) = tokio::sync::mpsc::channel(128);
    let (input_ev_tx, mut input_ev_rx) = tokio::sync::mpsc::channel(128);
    let (mut message_tx, mut message_rx) = tokio::sync::mpsc::channel(128);

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


    let mut state = State::new();
    let mut global_scope = Arc::new(tokio::sync::Mutex::new(mappings::bind_mappings(&mut state)));
    let mut window_cycle_token: usize = 0;

    let mut mappings = CompiledKeyMappings::new();

    {
        let mut message_tx = message_tx.clone();
        let mut global_scope = global_scope.clone();
        task::spawn(async move {
            eval_block(&mut *global_scope.lock().await, &mut Ambient {
                window_cycle_token,
                message_tx: Some(&mut message_tx),
            }).await;
        });
    }

    fn handle_active_window_change(block: &mut Arc<tokio::sync::Mutex<Block>>, message_tx: &mut ExecutionMessageSender, mappings: &mut CompiledKeyMappings, window_cycle_token: usize) {
        *mappings = CompiledKeyMappings::new();

        let mut message_tx = message_tx.clone();
        let mut block = block.clone();
        task::spawn(async move {
            eval_block(&mut *block.lock().await,
                       &mut Ambient {
                           message_tx: Some(&mut message_tx),
                           window_cycle_token,
                       },
            ).await;
        });
    }

    fn handle_execution_message(current_token: usize, msg: ExecutionMessage, state: &mut State, mappings: &mut CompiledKeyMappings) {
        match msg {
            ExecutionMessage::EatEv(action) => {
                state.ignore_list.ignore(&action);
            }
            ExecutionMessage::AddMapping(token, from, mut block) => {
                if token == current_token {
                    mappings.0.insert(from, Arc::new(tokio::sync::Mutex::new(block)));
                }
            }
        }
    }

    loop {
        tokio::select! {
            Some(window) = window_ev_rx.recv() => {
                state.active_window = Some(window);
                handle_active_window_change(&mut global_scope, &mut message_tx, &mut mappings, window_cycle_token + 1);
            }
            Some(ev) = input_ev_rx.recv() => {
                handle_stdin_ev(&mut state, &ev, &mut mappings, &mut message_tx, window_cycle_token).unwrap();
            }
            Some(msg) = message_rx.recv() => {
                handle_execution_message(window_cycle_token, msg, &mut state, &mut mappings);
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


fn update_modifiers(state: &mut State, ev: &input_event) {
    vec![
        (INPUT_EV_LEFTMETA, state.meta_is_down),
        // (INPUT_EV_RIGHTMETA, ModifierName::RightMeta),
    ]
        .iter_mut()
        .for_each(|(a, b)| {
            if *ev == a.down {
                // *state.get_modifier_state(b) = true;
                *b = true;
            } else if *ev == a.up {
                // *state.get_modifier_state(b) = false;
                *b = false;
                if state.ignore_list.is_ignored(&KeyAction::new(a.to_key(), TYPE_UP)) {
                    state.ignore_list.unignore(&KeyAction::new(a.to_key(), TYPE_UP));
                    return;
                }
            }
        });
}

fn handle_stdin_ev(mut state: &mut State, ev: &input_event, mappings: &mut CompiledKeyMappings, message_tx: &mut ExecutionMessageSender, window_cycle_token: usize) -> Result<()> {
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

    let from_key_action = KeyActionWithMods {
        key: Key { key_type: ev.type_ as i32, code: ev.code as i32 },
        value: ev.value,
        modifiers: from_modifiers,
    };

    if let Some(block) = mappings.0.get(&from_key_action) {
        let block = block.clone();
        let mut message_tx = message_tx.clone();
        task::spawn(async move {
            let block_guard = block.lock().await;
            eval_block(&*block_guard, &mut Ambient { message_tx: Some(&mut message_tx), window_cycle_token }).await;
        });
        return Ok(());
    }

    print_event(&ev);

    Ok(())
}