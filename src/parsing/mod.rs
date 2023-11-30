use anyhow::*;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res, opt, recognize};
use nom::Err as NomErr;
use nom::IResult;
use nom::multi::{many0, many1};
use nom::sequence::terminated;
use tap::Tap;

use custom_combinators::*;
use error::*;
use identifier::*;
use key::*;
use key_action::*;
pub use key_action::{ParsedKeyAction, ParsedKeyActionVecExt};
use key_sequence::*;
use motion_action::*;
pub use public_parsing_api::*;

use crate::*;

mod custom_combinators;
mod identifier;
mod key;
mod motion_action;
mod action_state;
mod key_action;
mod key_sequence;
mod error;
mod public_parsing_api;

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
        Err(err) => { panic!("got other nom error: {}", err) }
        Ok((rest, res)) => { panic!("expected nom error, but got Ok\nresult: {:?}\nrest: '{}'\n", res, rest) }
    }
}

#[cfg(test)]
pub(super) fn nom_ok_rest<T>(rest: &str, value: T) -> ParseResult<&str, T> { Ok((rest, value)) }