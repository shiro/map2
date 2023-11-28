#![feature(fn_traits)]
#![feature(type_alias_impl_trait)]

#![recursion_limit = "256"]

#![allow(warnings)]

extern crate core;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::{fs, io};
use std::borrow::BorrowMut;
// #[macro_use]
// use subscriber::linkable;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, mpsc, Mutex, RwLock, Weak};
use std::thread;
use std::time::Duration;

pub use anyhow::{anyhow, Result};
use arc_swap::ArcSwapOption;
use evdev_rs::enums::EventCode;
pub use evdev_rs::InputEvent as EvdevInputEvent;
use nom::lib::std::collections::HashMap;
use tap::Tap;
use uuid::Uuid;

use event_loop::EVENT_LOOP;
pub use mapper::mapper::Mapper;
pub use python::err_to_py;
use reader::Reader;
use subscriber_map::SubscriberMap;
use writer::Writer;

use crate::device::virtual_input_device::grab_udev_inputs;
use crate::error::*;
use crate::event::InputEvent;
pub use crate::key_defs::*;
use crate::key_primitives::*;
use crate::state::*;

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
pub mod global;
pub mod platform;
pub mod subscriber_map;

#[cfg(feature = "integration")]
pub mod testing;


pub mod python;
pub mod reader;
pub mod virtual_reader;
pub mod mapper;
pub mod writer;
pub mod virtual_writer;
pub mod text_mapper;
pub mod window;