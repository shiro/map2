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
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex, RwLock, Weak, mpsc};
use std::thread;
use std::time::Duration;
use std::{fs, io};

pub use evdev_rs::enums::EV_ABS::*;
pub use evdev_rs::enums::EV_KEY::*;
pub use evdev_rs::enums::EV_REL::*;
pub use key_primitives::Key;
pub use parsing::*;

pub use anyhow::{Result, anyhow};
pub use evdev_rs::InputEvent as EvdevInputEvent;
use evdev_rs::enums::EventCode;
use nom::lib::std::collections::{BTreeSet, HashMap, HashSet};
use tap::Tap;
use uuid::Uuid;

pub use crate::closure_channel::*;
use crate::error::*;
use crate::event::InputEvent;
pub use crate::key_defs::*;
use crate::key_primitives::*;
use conversions::extract_with_error;
use event_loop::EVENT_LOOP;
pub use node::*;
pub use python::{PyBound, err_to_py};

pub mod capabilities;
pub mod closure_channel;
pub mod conversions;
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
pub mod xkb;
pub mod xkb_transformer_registry;

#[cfg(feature = "integration")]
pub mod testing;

pub mod node;
pub mod python;
pub mod python_util;
pub mod virtual_writer;
pub mod window;
