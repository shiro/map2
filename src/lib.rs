#![feature(fn_traits)]
#![recursion_limit = "256"]

#![allow(warnings)]

extern crate core;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::{fs, io};
use std::borrow::BorrowMut;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, mpsc, Mutex, RwLock, Weak};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use arc_swap::ArcSwapOption;
use evdev_rs::enums::EventCode;
use evdev_rs::InputEvent as EvdevInputEvent;
use nom::lib::std::collections::HashMap;
use thread_tryjoin::TryJoinHandle;
use uuid::Uuid;

use event_loop::EVENT_LOOP;
pub use mapper::mapper::Mapper;
pub use python::err_to_py;
use reader::Reader;
use writer::Writer;

use crate::device::virtual_input_device::grab_udev_inputs;
use crate::event::InputEvent;
use crate::key_defs::*;
use crate::key_primitives::*;
use crate::state::*;
use crate::x11::ActiveWindowInfo;
use crate::error::*;

// #[macro_use]
// use subscriber::linkable;

pub mod x11;
pub mod key_defs;
pub mod state;
pub mod key_primitives;
pub mod parsing;
pub mod device;
pub mod event_handlers;
pub mod logging;
pub mod event_loop;
pub mod event;
pub mod subscriber;
pub mod encoding;
pub mod xkb;
pub mod xkb_transformer_registry;
pub mod error;

#[cfg(test)]
pub mod tests;


pub mod python;
pub mod reader;
pub mod virtual_reader;
pub mod mapper;
pub mod writer;
pub mod virtual_writer;
pub mod text_mapper;
pub mod window;