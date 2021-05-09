#![feature(type_ascription)]
#![feature(async_closure)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(label_break_value)]
#![feature(destructuring_assignment)]
#![feature(seek_convenience)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

pub use std::{io, time};
pub use std::io::Write;
pub use std::ops::DerefMut;
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
pub use crate::scope::*;
pub use crate::state::*;
pub use crate::x11::{x11_initialize, x11_test};
pub use crate::x11::ActiveWindowInfo;

pub mod tab_mod;
pub mod caps_mod;
pub mod rightalt_mod;
pub mod x11;
pub mod key_defs;
pub mod state;
pub mod scope;
pub mod mappings;
pub mod block_ext;
pub mod key_primitives;
pub mod parsing;
pub mod device;
pub mod cli;
pub mod test;
pub mod ignore_list;
pub mod messaging;
