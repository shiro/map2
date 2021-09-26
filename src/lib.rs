#[macro_use]
extern crate lazy_static;
extern crate regex;

pub use std::{io, time, fs};
pub use std::borrow::BorrowMut;
pub use std::io::Write;
pub use std::ops::{Deref, DerefMut};
pub use std::sync::Arc;
pub use std::sync::Mutex;

pub use anyhow::{anyhow, Result};
pub use async_recursion::async_recursion;
pub use evdev_rs::enums::EventCode;
pub use evdev_rs::InputEvent;
pub use nom::lib::std::collections::HashMap;
pub use tokio::prelude::*;
pub use tokio::sync::{mpsc, oneshot};
pub use tokio::task;

pub use crate::cli::parse_cli;
pub use crate::device::virtual_input_device::bind_udev_inputs;
pub use crate::key_defs::*;
pub use crate::key_primitives::*;
pub use crate::runtime::*;
pub use crate::runtime::evaluation::*;
pub use crate::state::*;
pub use crate::x11::{x11_initialize, get_window_info_x11};
pub use crate::x11::ActiveWindowInfo;

pub mod x11;
pub mod key_defs;
pub mod state;
pub mod runtime;
pub mod script;
pub mod block_ext;
pub mod key_primitives;
pub mod parsing;
pub mod device;
pub mod cli;
pub mod ignore_list;
pub mod messaging;
pub mod event_handlers;
pub mod logging;

#[cfg(test)]
pub mod tests;


pub mod python;
pub mod python_reader;
pub mod python_writer;
pub mod python_window;
