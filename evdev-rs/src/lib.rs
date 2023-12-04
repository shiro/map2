//! Rust bindings to libevdev, a wrapper for evdev devices.
//!
//! This library intends to provide a safe interface to the libevdev library. It
//! will look for the library on the local system, and link to the installed copy.
//!
//! # Examples
//!
//! ## Intializing a evdev device
//!
//! ```rust,no_run
//! use evdev_rs::Device;
//! use evdev_rs::device::File;
//!
//! let file = File::open("/dev/input/event0").unwrap();
//! let mut d = Device::new_from_file(file).unwrap();
//! ```
//!
//! ## Getting the next event
//!
//! ```rust,no_run
//! use evdev_rs::Device;
//! use evdev_rs::device::File;
//! use evdev_rs::ReadFlag;
//!
//! let file = File::open("/dev/input/event0").unwrap();
//! let mut d = Device::new_from_file(file).unwrap();
//!
//! loop {
//!     let ev = d.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);
//!     match ev {
//!         Ok(ev) => println!("Event: time {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
//!                           ev.time.tv_sec,
//!                           ev.time.tv_usec,
//!                           ev.event_type().map(|ev_type| format!("{}", ev_type)).unwrap_or("".to_owned())),
//!         Err(e) => (),
//!     }
//! }
//! ```
//!
//! ## Serialization
//! to use serialization, you muse enable the `serde` feature.
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! evdev-rs = { version = "0.4.0", features = ["serde"] }
//! ```

#[macro_use]
mod macros;
pub mod device;
pub mod enums;
pub mod logging;
pub mod uinput;
pub mod util;

use bitflags::bitflags;
use libc::{c_uint, suseconds_t, time_t};
use std::convert::{TryFrom, TryInto};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use enums::*;
use util::*;

use evdev_sys as raw;

#[doc(inline)]
pub use device::Device;
#[doc(inline)]
pub use device::DeviceWrapper;
#[doc(inline)]
pub use device::UninitDevice;
#[doc(inline)]
pub use uinput::UInputDevice;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub enum GrabMode {
    /// Grab the device if not currently grabbed
    Grab = raw::LIBEVDEV_GRAB as isize,
    /// Ungrab the device if currently grabbed
    Ungrab = raw::LIBEVDEV_UNGRAB as isize,
}

bitflags! {
    pub struct ReadFlag: u32 {
        /// Process data in sync mode
        const SYNC = 1;
        /// Process data in normal mode
        const NORMAL = 2;
        /// Pretend the next event is a SYN_DROPPED and require the
        /// caller to sync
        const FORCE_SYNC = 4;
        /// The fd is not in O_NONBLOCK and a read may block
        const BLOCKING = 8;
    }
}

#[derive(PartialEq)]
pub enum ReadStatus {
    /// `next_event` has finished without an error and an event is available
    /// for processing.
    Success = raw::LIBEVDEV_READ_STATUS_SUCCESS as isize,
    /// Depending on the `next_event` read flag:
    /// libevdev received a SYN_DROPPED from the device, and the caller should
    /// now resync the device, or, an event has been read in sync mode.
    Sync = raw::LIBEVDEV_READ_STATUS_SYNC as isize,
}

pub enum LedState {
    /// Turn the LED on
    On = raw::LIBEVDEV_LED_ON as isize,
    /// Turn the LED off
    Off = raw::LIBEVDEV_LED_OFF as isize,
}

pub struct DeviceId {
    pub bustype: BusType,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

/// used by EVIOCGABS/EVIOCSABS ioctls
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AbsInfo {
    /// latest reported value for the axis
    pub value: i32,
    /// specifies minimum value for the axis
    pub minimum: i32,
    /// specifies maximum value for the axis
    pub maximum: i32,
    /// specifies fuzz value that is used to filter noise from
    /// the event stream
    pub fuzz: i32,
    /// values that are within this value will be discarded by
    /// joydev interface and reported as 0 instead
    pub flat: i32,
    /// specifies resolution for the values reported for
    /// the axis
    pub resolution: i32,
}

impl AbsInfo {
    pub fn from_raw(absinfo: libc::input_absinfo) -> AbsInfo {
        AbsInfo {
            value: absinfo.value,
            minimum: absinfo.minimum,
            maximum: absinfo.maximum,
            fuzz: absinfo.fuzz,
            flat: absinfo.flat,
            resolution: absinfo.resolution,
        }
    }

    pub fn as_raw(&self) -> libc::input_absinfo {
        libc::input_absinfo {
            value: self.value,
            minimum: self.minimum,
            maximum: self.maximum,
            fuzz: self.fuzz,
            flat: self.flat,
            resolution: self.resolution,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize), derive(Deserialize))]
#[derive(Copy, Clone, Eq, Hash, PartialOrd, Ord, Debug, PartialEq, Default)]
pub struct TimeVal {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

impl TryFrom<SystemTime> for TimeVal {
    type Error = SystemTimeError;
    fn try_from(system_time: SystemTime) -> Result<Self, Self::Error> {
        let d = system_time.duration_since(UNIX_EPOCH)?;
        Ok(TimeVal {
            tv_sec: d.as_secs() as time_t,
            tv_usec: d.subsec_micros() as suseconds_t,
        })
    }
}

impl TryInto<SystemTime> for TimeVal {
    type Error = ();
    /// Fails if TimeVal.tv_usec is >= 10^6 or if the TimeVal is outside
    /// the range of SystemTime
    fn try_into(self) -> Result<SystemTime, Self::Error> {
        let secs = self.tv_sec.try_into().map_err(|_| ())?;
        let nanos = (self.tv_usec * 1000).try_into().map_err(|_| ())?;
        let duration = Duration::new(secs, nanos);
        UNIX_EPOCH.checked_add(duration).ok_or(())
    }
}

impl TimeVal {
    pub fn new(tv_sec: time_t, tv_usec: suseconds_t) -> TimeVal {
        const MICROS_PER_SEC: suseconds_t = 1_000_000;
        TimeVal {
            tv_sec: tv_sec + tv_usec / MICROS_PER_SEC,
            tv_usec: tv_usec % MICROS_PER_SEC,
        }
    }

    pub fn from_raw(timeval: &libc::timeval) -> TimeVal {
        TimeVal {
            tv_sec: timeval.tv_sec,
            tv_usec: timeval.tv_usec,
        }
    }

    pub fn as_raw(&self) -> libc::timeval {
        libc::timeval {
            tv_sec: self.tv_sec,
            tv_usec: self.tv_usec,
        }
    }
}

/// The event structure itself
#[cfg_attr(feature = "serde", derive(Serialize), derive(Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputEvent {
    /// The time at which event occured
    pub time: TimeVal,
    pub event_code: EventCode,
    pub value: i32,
}

impl InputEvent {
    pub fn new(timeval: &TimeVal, code: &EventCode, value: i32) -> InputEvent {
        InputEvent {
            time: *timeval,
            event_code: *code,
            value,
        }
    }

    pub fn event_type(&self) -> Option<EventType> {
        int_to_event_type(event_code_to_int(&self.event_code).0)
    }

    pub fn from_raw(event: &libc::input_event) -> InputEvent {
        let ev_type = event.type_ as u32;
        let event_code = int_to_event_code(ev_type, event.code as u32);
        InputEvent {
            time: TimeVal::from_raw(&event.time),
            event_code,
            value: event.value,
        }
    }

    pub fn as_raw(&self) -> libc::input_event {
        let (ev_type, ev_code) = event_code_to_int(&self.event_code);
        libc::input_event {
            time: self.time.as_raw(),
            type_: ev_type as u16,
            code: ev_code as u16,
            value: self.value,
        }
    }

    pub fn is_type(&self, ev_type: &EventType) -> bool {
        unsafe { raw::libevdev_event_is_type(&self.as_raw(), *ev_type as c_uint) == 1 }
    }

    pub fn is_code(&self, code: &EventCode) -> bool {
        let (ev_type, ev_code) = event_code_to_int(code);

        unsafe { raw::libevdev_event_is_code(&self.as_raw(), ev_type, ev_code) == 1 }
    }
}
