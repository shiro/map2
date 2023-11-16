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


#[cfg(test)]
pub(super) fn nom_ok<'a, T>(value: T) -> ResNew<&'a str, T> { Ok(("", (value, None))) }

#[cfg(test)]
pub(super) fn nom_err<I, T>(rest: I, expected: Vec<String>) -> ResNew<I, T> {
    Err(NomErr::Error(CustomError {
        input: rest,
        expected,
    }))
}

#[cfg(test)]
pub(super) fn assert_nom_err<T>(parse_result: ResNew<&str, T>, rest: &str) {
    // if !matches!(parse_result, Err(NomErr::Error(CustomError { input: "bb", .. }))){
    match parse_result {
        Err(NomErr::Error(x)) => {
            assert_eq!(x.input, rest);
        }
        Err(err) => { panic!("got other nom error: {err}") }
        Ok((rest, err)) => { panic!("expected nom error, but got Ok\nrest: {}", rest) }
    }
    // }
}

#[cfg(test)]
pub(super) fn nom_ok_rest<T>(rest: &str, value: T) -> ResNew<&str, T> { Ok((rest, (value, None))) }

#[cfg(test)]
pub(super) fn nom_eval<'a, T>(value: ResNew<&str, T>) -> T { value.unwrap().1.0 }

#[cfg(test)]
pub(super) fn nom_no_last_err<'a, T>(value: ResNew<&str, T>) -> ResNew<&str, T> {
    match value {
        Ok((input, (val, _))) => Ok((input, (val, None))),
        Err(err) => Err(err)
    }
}

use crate::*;

mod custom_combinators;
mod identifier;
mod key;
pub mod key_action;
mod key_sequence;
mod error;
pub(crate) mod python;
