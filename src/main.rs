mod tab_mod;
mod caps_mod;
mod leftalt_mod;
mod rightalt_mod;

use std::process::exit;
use std::{io, mem, slice, thread, time};
use std::io::{Read, stdout, Write};

use anyhow::Result;
use input_linux_sys::{KEY_E, KEY_K, KEY_J, EV_KEY, KEY_TAB, KEY_LEFTMETA, KEY_LEFTSHIFT, KEY_LEFTALT, EV_SYN, SYN_REPORT, EV_MSC, MSC_SCAN, KEY_CAPSLOCK, KEY_LEFTCTRL, KEY_ESC, KEY_H, KEY_L, KEY_LEFT, KEY_DOWN, KEY_RIGHT, KEY_UP, KEY_RIGHTALT, KEY_F8};
use std::borrow::{BorrowMut, Borrow};

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
        stdout().write_all(any_as_u8_slice(ev));
        // println!("{:?}", ev);
        stdout().flush();
    }
}

fn equal(ev1: &input_event, ev2: &input_event) -> bool {
    ev1.code == ev2.code &&
        ev1.type_ == ev2.type_ &&
        ev1.value == ev2.value
}


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

static SYN: input_event = input_event { type_: EV_SYN as u16, code: SYN_REPORT as u16, value: 0, time: DUMMY_TIME };


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

fn main() -> Result<()> {
    let mut stdin = io::stdin();

    let mut ev: input_event = unsafe { mem::zeroed() };

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
        unsafe {
            let slice = any_as_u8_slice_mut(&mut ev);
            match stdin.read_exact(slice) {
                Ok(()) => (),
                Err(e) => {
                    ::std::mem::forget(slice);
                    panic!();
                }
            }
        }

        if ev.type_ == EV_SYN as u16 || ev.type_ == EV_MSC as u16 && ev.code == MSC_SCAN as u16 {
            print_event(&ev);
            continue;
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


        if crate::tab_mod::tab_mod(&ev, &mut state) {
            continue;
        }

        if !state.leftcontrol_is_down {
            if crate::caps_mod::caps_mod(&ev, &mut state) {
                continue;
            }
        }

        if !state.disable_alt_mod {
            if crate::leftalt_mod::leftalt_mod(&ev, &mut state) {
                continue;
            }
        }

        if !state.disable_alt_mod {
            if crate::rightalt_mod::rightalt_mod(&ev, &mut state) {
                continue;
            }
        }

        print_event(&ev);
    }

    Ok(())
}