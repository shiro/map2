use anyhow::*;
use evdev_rs::enums::EventType;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res, opt, recognize};
use nom::Err as NomErr;
use nom::IResult;
use nom::multi::{many0, many1};
use nom::sequence::terminated;
use tap::Tap;

use motion_action::*;
use custom_combinators::*;
use error::*;
use identifier::*;
use key::*;
use key_action::*;
use key_sequence::*;

use crate::*;

mod custom_combinators;
mod identifier;
mod key;
mod motion_action;
pub mod action_state;
pub mod key_action;
mod key_sequence;
mod error;
pub mod python;


#[cfg(test)]
pub(super) fn nom_ok<'a, T>(value: T) -> ParseResult<&'a str, T> { Ok(("", value)) }

#[cfg(test)]
pub(super) fn nom_err<I, T>(rest: I, expected: Vec<String>) -> ParseResult<I, T> {
    Err(NomErr::Error(CustomError {
        input: rest,
        expected,
    }))
}

#[cfg(test)]
pub(super) fn assert_nom_err<T: std::fmt::Debug>(parse_result: ParseResult<&str, T>, rest: &str) {
    match parse_result {
        Err(NomErr::Error(x)) => {
            assert_eq!(x.input, rest);
        }
        Err(err) => { panic!("got other nom error: {err}") }
        Ok((rest, res)) => { panic!("expected nom error, but got Ok\nresult: {:?}\nrest: '{}'\n", res, rest) }
    }
}

#[cfg(test)]
pub(super) fn nom_ok_rest<T>(rest: &str, value: T) -> ParseResult<&str, T> { Ok((rest, value)) }