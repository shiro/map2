// use input_linux_sys::input_event;
// use input_linux_sys::__u16;
// use input_linux_sys::__s32;

// use nom::{
//     IResult,
//     bytes::complete::{tag, take_while_m_n},
//     combinator::map_res,
//     sequence::tuple,
// };
use std::process::exit;
use std::{io, mem, slice, thread, time};
use std::io::{Read, stdout, Write};

use anyhow::Result;
use input_linux_sys::{KEY_E, KEY_K, KEY_J, EV_KEY, KEY_TAB, KEY_LEFTMETA, KEY_LEFTSHIFT, KEY_LEFTALT, EV_SYN, SYN_REPORT, EV_MSC, MSC_SCAN};
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

struct State {
    tab_is_down: bool,
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

static ALT_UP: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 0, time: DUMMY_TIME };
static ALT_DOWN: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 1, time: DUMMY_TIME };
static ALT_REPEAT: input_event = input_event { type_: EV_KEY as u16, code: KEY_LEFTALT as u16, value: 2, time: DUMMY_TIME };

static SYN: input_event = input_event { type_: EV_SYN as u16, code: SYN_REPORT as u16, value: 0, time: DUMMY_TIME };


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

fn tab_mod(ev: &input_event, state: &mut State) -> bool {
    if state.tab_is_down {
        // tab repeat
        if equal(&ev, &TAB_DOWN) || equal(&ev, &TAB_REPEAT) {
            return true;
        }

        // tab up
        if equal(&ev, &TAB_UP) {
            state.tab_is_down = false;

            // tab up was handled before, just release all mods
            if ev_ignored(&TAB_DOWN, &mut state.ignore_list) {
                unignore_ev(&TAB_DOWN, &mut state.ignore_list);
                print_event(&ALT_UP);
                print_event(&SHIFT_UP);
                print_event(&META_UP);
                return true;
            }

            print_event(&TAB_DOWN);
            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&TAB_UP);
            return true;
        }

        // tab + [key down]
        if ev.value == 1 {
            ignore_ev(&TAB_DOWN, &mut state.ignore_list);
            print_event(&ALT_DOWN);
            print_event(&SHIFT_DOWN);
            print_event(&META_DOWN);
            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
        }
    } else if equal(&ev, &TAB_DOWN) {
        state.tab_is_down = true;
        return true;
    }

    false
}

fn main() -> Result<()> {
    let mut stdin = io::stdin();

    let mut ev: input_event = unsafe { mem::zeroed() };

    let mut state = State {
        tab_is_down: false,
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

        if tab_mod(&ev, &mut state) {
            continue;
        }

        print_event(&ev);
    }

    Ok(())
}