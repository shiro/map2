use anyhow::*;
use evdev_rs::enums::EventType;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::map;
use nom::Err as NomErr;
use nom::IResult;
use nom::multi::many0;
use nom::sequence::*;
use tap::Tap;

use custom_combinators::*;
use error::*;
use identifier::*;
use key::*;
use key_action::*;
use key_sequence::*;
#[cfg(test)]
use tests::*;

use crate::*;

mod custom_combinators;
mod identifier;
mod key;
pub mod key_action;
mod key_sequence;
mod error;
pub(crate) mod python;
