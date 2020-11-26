#![feature(type_ascription)]
#![feature(async_closure)]

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
use std::io::{Read, stdout, Write};

// use anyhow::Result;
use input_linux_sys::{KEY_E, KEY_K, KEY_J, EV_KEY, KEY_TAB, KEY_LEFTMETA, KEY_LEFTSHIFT, KEY_LEFTALT, EV_SYN, SYN_REPORT, EV_MSC, MSC_SCAN, KEY_CAPSLOCK, KEY_LEFTCTRL, KEY_ESC, KEY_H, KEY_L, KEY_LEFT, KEY_DOWN, KEY_RIGHT, KEY_UP, KEY_RIGHTALT, KEY_F8, REL_Y, REL_X, EV_REL};
use std::borrow::{BorrowMut, Borrow};
use crate::x11::{x11_get_active_window, x11_test, x11_test_async};
use tokio::task;
// use std::error::Error;
use anyhow::Result;
use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio::stream::StreamExt;

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
        // stdout().write_all(any_as_u8_slice(ev));
        if ev.type_ == EV_KEY as u16 {
            println!("{:?}", ev);

            // let res = x11_get_active_window().unwrap();
            // println!("class: {}", res.class);

            stdout().flush();
        }
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

#[tokio::main]
async fn main() -> Result<()> {
// async fn main() -> () {
    // x11_test()?;
    // x11_test_async().await?;

    let (mut tx1, mut rx1) = tokio::sync::mpsc::channel(128);

    let task1 = tokio::spawn(async move {
        loop {
            let res = task::spawn_blocking(move || {
                x11_test().ok()?
            }).await;

            if let Ok(Some(val)) = res {
                tx1.send(val).await;
                // println!("class: {}", val.class);
            }
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
    };

    loop {
        tokio::select! {
            Some(v) = rx1.recv() => {
                println!("class: {}", v.class);
            }
            _ = handle_stdin_ev(&mut state) => {}
            else => { break }
        }
    }

    Ok(())
}

async fn handle_stdin_ev(state: &mut State) -> Result<()> {
    let mut stdin = tokio::io::stdin();
    let mut ev: input_event = unsafe { mem::zeroed() };

    unsafe {
        let slice = any_as_u8_slice_mut(&mut ev);
        match stdin.read_exact(slice).await {
            Ok(_) => (),
            Err(e) => {
                ::std::mem::forget(slice);
                panic!();
            }
        }
    }

    if ev.type_ == EV_SYN as u16 || ev.type_ == EV_MSC as u16 && ev.code == MSC_SCAN as u16 {
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

    // if ev.type_ == EV_REL as u16 && ev.code == REL_X as u16 {
    //     println!("{}", ev.value);
    // }

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

    if ev.type_ == EV_KEY as u16 && ev.code == MOUSE7_DOWN.code as u16 {
        print_event(&META_DOWN);
        return Ok(());
    }
    if ev.type_ == EV_KEY as u16 && ev.code == MOUSE7_REPEAT.code as u16 {
        return Ok(());
    }
    if ev.type_ == EV_KEY as u16 && ev.code == MOUSE7_UP.code as u16 {
        print_event(&META_UP);
        return Ok(());
    }

    print_event(&ev);

    Ok(())
}