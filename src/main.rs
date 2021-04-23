#![feature(type_ascription)]
#![feature(async_closure)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(label_break_value)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::{io, time};
use std::io::{Write};
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use evdev_rs::enums::EventCode;
use evdev_rs::InputEvent;
use nom::lib::std::collections::HashMap;
use tokio::prelude::*;
use tokio::sync::{mpsc, oneshot};
use tokio::task;

use crate::device::device_test::bind_udev_inputs;
use crate::key_defs::*;
use crate::key_primitives::*;
use crate::scope::*;
use crate::state::*;
use crate::x11::{x11_initialize, x11_test};
use crate::x11::ActiveWindowInfo;

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
mod parsing;
mod device;
mod udevmon;


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

fn log_msg(msg: &str) {
    let out_msg = format!("[DEBUG] {}\n", msg);

    io::stderr().write_all(out_msg.as_bytes()).unwrap();
}


#[derive(Debug)]
pub(crate) enum ExecutionMessage {
    EatEv(KeyAction),
    AddMapping(usize, KeyActionWithMods, Block),
    GetFocusedWindowInfo(tokio::sync::mpsc::Sender<Option<ActiveWindowInfo>>),
}

pub(crate) type ExecutionMessageSender = tokio::sync::mpsc::Sender<ExecutionMessage>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stdin = tokio::io::stdin();
    // let mut read_ev: input_event = unsafe { mem::zeroed() };

    let (window_ev_tx, mut window_ev_rx) = tokio::sync::mpsc::channel(128);
    // let (input_ev_tx, mut input_ev_rx) = tokio::sync::mpsc::channel(128);
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


    let mut state = State::new();
    let mut global_scope = Arc::new(tokio::sync::Mutex::new(mappings::bind_mappings()));
    let mut window_cycle_token: usize = 0;
    let mut mappings = CompiledKeyMappings::new();


    // experimental udev stuff
    let patterns = vec![
        "/dev/input/by-id/.*-event-mouse",
        "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd"
    ];

    let (mut ev_reader_init_tx, mut ev_reader_init_rx) = oneshot::channel();
    let (mut ev_writer_tx, mut ev_writer_rx) = mpsc::channel(128);
    // start coroutine
    bind_udev_inputs(patterns, ev_reader_init_tx, ev_writer_tx).await;
    let mut ev_reader_tx = ev_reader_init_rx.await.unwrap();


    {
        let mut message_tx = message_tx.clone();
        let global_scope = global_scope.clone();
        let ev_reader_tx = ev_reader_tx.clone();
        task::spawn(async move {
            eval_block(&mut *global_scope.lock().await, &mut Ambient {
                ev_writer_tx: ev_reader_tx,
                window_cycle_token,
                message_tx: Some(&mut message_tx),
            }).await;
        });
    }

    fn handle_active_window_change(block: &mut Arc<tokio::sync::Mutex<Block>>, ev_writer_tx: &mut mpsc::Sender<InputEvent>, message_tx: &mut ExecutionMessageSender, mappings: &mut CompiledKeyMappings, window_cycle_token: usize) {
        *mappings = CompiledKeyMappings::new();

        let mut message_tx = message_tx.clone();
        let mut block = block.clone();
        let ev_writer_tx = ev_writer_tx.clone();
        task::spawn(async move {
            eval_block(&mut *block.lock().await,
                       &mut Ambient {
                           ev_writer_tx,
                           message_tx: Some(&mut message_tx),
                           window_cycle_token,
                       },
            ).await;
        });
    }

    async fn handle_execution_message(current_token: usize, msg: ExecutionMessage, state: &mut State, mappings: &mut CompiledKeyMappings) {
        match msg {
            ExecutionMessage::EatEv(action) => {
                state.ignore_list.ignore(&action);
            }
            ExecutionMessage::AddMapping(token, from, block) => {
                if token == current_token {
                    mappings.0.insert(from, Arc::new(tokio::sync::Mutex::new(block)));
                }
            }
            ExecutionMessage::GetFocusedWindowInfo(tx) => {
                tx.send(state.active_window.clone()).await.unwrap();
            }
        }
    }

    loop {
        tokio::select! {
            Some(window) = window_ev_rx.recv() => {
                state.active_window = Some(window);
                window_cycle_token = window_cycle_token + 1;
                handle_active_window_change(&mut global_scope, &mut ev_reader_tx, &mut message_tx, &mut mappings, window_cycle_token);
            }
            Some(ev) = ev_writer_rx.recv() => {
                handle_stdin_ev(&mut state, ev, &mut mappings, &mut ev_reader_tx, &mut message_tx, window_cycle_token).await.unwrap();
            }
            Some(msg) = message_rx.recv() => {
                handle_execution_message(window_cycle_token, msg, &mut state, &mut mappings).await;
            }
            else => { break }
        }
    }

    Ok(())
}


// fn update_modifiers(state: &mut State, ev: &input_event) {
//     vec![
//         (INPUT_EV_LEFTMETA, state.meta_is_down),
//         // (INPUT_EV_RIGHTMETA, ModifierName::RightMeta),
//     ]
//         .iter_mut()
//         .for_each(|(a, b)| {
//             if *ev == a.down {
//                 // *state.get_modifier_state(b) = true;
//                 *b = true;
//             } else if *ev == a.up {
//                 // *state.get_modifier_state(b) = false;
//                 *b = false;
//                 if state.ignore_list.is_ignored(&KeyAction::new(a.to_key(), TYPE_UP)) {
//                     state.ignore_list.unignore(&KeyAction::new(a.to_key(), TYPE_UP));
//                     return;
//                 }
//             }
//         });
// }

async fn handle_stdin_ev(mut state: &mut State, ev: InputEvent, mappings: &mut CompiledKeyMappings, ev_writer: &mut mpsc::Sender<InputEvent>, message_tx: &mut ExecutionMessageSender, window_cycle_token: usize) -> Result<()> {
    match ev.event_code {
        EventCode::EV_KEY(_) => {}
        _ => {
            ev_writer.send(ev).await;
            return Ok(());
        }
    }

    // if ev.type_ != input_linux_sys::EV_KEY as u16 {
    //     // print_event(&ev);
    //     return Ok(());
    // }

    // update_modifiers(&mut state, &ev);

    // if crate::tab_mod::tab_mod(&ev, &mut *state) {
    //     return Ok(());
    // }
    //
    // if !state.leftcontrol_is_down {
    //     if crate::caps_mod::caps_mod(&ev, &mut *state) {
    //         return Ok(());
    //     }
    // }
    //
    // if !state.disable_alt_mod {
    //     if crate::rightalt_mod::rightalt_mod(&ev, &mut *state) {
    //         return Ok(());
    //     }
    // }

    let mut from_modifiers = KeyModifierFlags::new();
    from_modifiers.ctrl = state.leftcontrol_is_down.clone();
    from_modifiers.alt = state.leftalt_is_down.clone();
    from_modifiers.shift = state.shift_is_down.clone();
    from_modifiers.meta = state.meta_is_down.clone();

    let from_key_action = KeyActionWithMods {
        key: Key { event_code: ev.event_code },
        value: ev.value,
        modifiers: from_modifiers,
    };

    if let Some(block) = mappings.0.get(&from_key_action) {
        let block = block.clone();
        let mut message_tx = message_tx.clone();
        let ev_writer = ev_writer.clone();
        task::spawn(async move {
            let block_guard = block.lock().await;
            eval_block(&*block_guard, &mut Ambient { ev_writer_tx: ev_writer, message_tx: Some(&mut message_tx), window_cycle_token }).await;
        });
        return Ok(());
    }

    ev_writer.send(ev).await;

    Ok(())
}

// input ev thread
// tokio::spawn(async move {
//     loop {
//         listen_to_key_events(&mut read_ev, &mut stdin).await;
//         input_ev_tx.send(read_ev).await.unwrap();
//     }
// });
