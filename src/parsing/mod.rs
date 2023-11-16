use anyhow::*;
use evdev_rs::enums::EventType;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, map_res, recognize};
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
use key_sequence::*;

use crate::*;

#[cfg(test)]
pub(super) fn nom_ok<'a, T>(value: T) -> ResNew2<&'a str, T> { Ok(("", value)) }

#[cfg(test)]
pub(super) fn nom_err<I, T>(rest: I, expected: Vec<String>) -> ResNew2<I, T> {
    Err(NomErr::Error(CustomError {
        input: rest,
        expected,
    }))
}

#[cfg(test)]
pub(super) fn assert_nom_err<T: std::fmt::Debug>(parse_result: ResNew2<&str, T>, rest: &str) {
    // if !matches!(parse_result, Err(NomErr::Error(CustomError { input: "bb", .. }))){
    match parse_result {
        Err(NomErr::Error(x)) => {
            assert_eq!(x.input, rest);
        }
        Err(err) => { panic!("got other nom error: {err}") }
        Ok((rest, res)) => { panic!("expected nom error, but got Ok\nresult: {:?}\nrest: '{}'\n", res, rest) }
    }
    // }
}

#[cfg(test)]
pub(super) fn nom_ok_rest<T>(rest: &str, value: T) -> ResNew2<&str, T> { Ok((rest, value)) }

#[cfg(test)]
pub(super) fn nom_eval<'a, T>(value: ResNew<&str, T>) -> T { value.unwrap().1.0 }

#[cfg(test)]
pub(super) fn nom_no_last_err<'a, T>(value: ResNew<&str, T>) -> ResNew<&str, T> {
    match value {
        Ok((input, (val, _))) => Ok((input, (val, None))),
        Err(err) => Err(err)
    }
}

mod custom_combinators;
mod identifier;
mod key;
pub mod key_action;
mod key_sequence;
mod error;
pub(crate) mod python;
