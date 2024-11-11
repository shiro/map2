#![feature(fn_traits)]
#![feature(type_alias_impl_trait)]
#![recursion_limit = "256"]
#![allow(warnings)]
#![feature(async_closure)]

extern crate core;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use arc_swap::ArcSwap;
use arc_swap::ArcSwapOption;
use std::borrow::BorrowMut;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::OnceLock;
use std::sync::{mpsc, Arc, Mutex, RwLock, Weak};
use std::thread;
use std::time::Duration;
use std::{fs, io};

pub use evdev_rs::enums::EV_ABS::*;
pub use evdev_rs::enums::EV_KEY::*;
pub use evdev_rs::enums::EV_REL::*;
pub use key_primitives::Key;
pub use parsing::*;

pub use anyhow::{anyhow, Result};
use evdev_rs::enums::EventCode;
pub use evdev_rs::InputEvent as EvdevInputEvent;
use nom::lib::std::collections::{BTreeSet, HashMap, HashSet};
use tap::Tap;
use uuid::Uuid;

use event_loop::EVENT_LOOP;
pub use mapper::*;
pub use python::{err_to_py, PyBound};
use reader::Reader;
pub use subscriber::*;
use writer::Writer;

pub use crate::closure_channel::*;
use crate::device::virtual_input_device::grab_udev_inputs;
use crate::error::*;
use crate::event::InputEvent;
pub use crate::key_defs::*;
use crate::key_primitives::*;

pub mod capabilities;
pub mod closure_channel;
pub mod device;
pub mod encoding;
pub mod error;
pub mod event;
pub mod event_handlers;
pub mod event_loop;
pub mod global;
pub mod key_defs;
pub mod key_primitives;
pub mod logging;
pub mod node_util;
pub mod parsing;
pub mod platform;
pub mod subscriber;
pub mod xkb;
pub mod xkb_transformer_registry;

#[cfg(feature = "integration")]
pub mod testing;

pub mod mapper;
pub mod python;
pub mod reader;
pub mod virtual_writer;
pub mod window;
pub mod writer;
