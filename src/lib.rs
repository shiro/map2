#![feature(fn_traits)]

extern crate core;
#[macro_use]
extern crate lazy_static;
extern crate regex;

pub use std::{fs, io, time};
pub use std::borrow::BorrowMut;
pub use std::io::Write;
pub use std::ops::{Deref, DerefMut};
pub use std::sync::Arc;
use std::sync::mpsc;
pub use std::sync::Mutex;
pub use std::time::Duration;

pub use anyhow::{anyhow, Result};
pub use async_recursion::async_recursion;
pub use evdev_rs::enums::EventCode;
pub use evdev_rs::InputEvent;
pub use nom::lib::std::collections::HashMap;
use thread_tryjoin::TryJoinHandle;
pub use uuid::Uuid;

pub use crate::device::virtual_input_device::grab_udev_inputs;
pub use crate::key_defs::*;
pub use crate::key_primitives::*;
pub use crate::state::*;
pub use crate::x11::{get_window_info_x11, x11_initialize};
pub use crate::x11::ActiveWindowInfo;

pub mod x11;
pub mod key_defs;
pub mod state;
pub mod key_primitives;
pub mod parsing;
pub mod device;
pub mod event_handlers;
pub mod logging;
pub mod event_loop;

#[cfg(test)]
pub mod tests;


pub mod python;
pub mod reader;
pub mod mapper;
pub mod writer;
pub mod text_mapper;
pub mod window;
