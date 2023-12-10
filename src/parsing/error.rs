use std::fmt::Debug;

use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::InputLength;

use super::*;

pub type ParseResult<I, O> = IResult<I, O, CustomError<I>>;

pub fn make_generic_nom_err_new<I>(input: I) -> NomErr<CustomError<I>> {
    NomErr::Error(CustomError {
        input,
        expected: vec![],
    })
}

pub fn make_generic_nom_err_options<I>(input: I, options: Vec<String>) -> NomErr<CustomError<I>> {
    NomErr::Error(CustomError {
        input,
        expected: options,
    })
}

#[derive(Debug, PartialEq)]
pub struct CustomError<I> {
    pub input: I,
    pub expected: Vec<String>,
}

impl<I> ParseError<I> for CustomError<I>
where
    I: InputLength,
{
    fn from_error_kind(input: I, _: ErrorKind) -> Self {
        CustomError {
            input,
            expected: vec![],
        }
    }

    fn from_char(input: I, ch: char) -> Self {
        CustomError {
            input,
            expected: vec![ch.to_string()],
        }
    }

    fn or(mut self, mut other: Self) -> Self {
        if other.input.input_len() < self.input.input_len() {
            return other;
        } else if other.input.input_len() > self.input.input_len() {
            return self;
        }
        other.expected.append(&mut self.expected);
        other
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for CustomError<I> {
    fn from_external_error(input: I, _: ErrorKind, _: E) -> Self {
        Self {
            input,
            expected: vec![],
        }
    }
}

pub trait FromTagError<I>: Sized {
    fn from_tag(input: I, tag: String) -> Self;
}

impl<Input> FromTagError<Input> for CustomError<Input> {
    fn from_tag(input: Input, tag: String) -> Self {
        Self {
            input,
            expected: vec![format!("'{}'", tag)],
        }
    }
}
