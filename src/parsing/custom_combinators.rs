use nom::{Compare, CompareResult, Err, InputIter, InputLength, InputTake, Parser, Slice, Offset};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{char, digit1, multispace0, multispace1};
use nom::combinator::{opt, recognize, value};
use nom::error::{ErrorKind, ParseError};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{pair, tuple};
use crate::parsing::error::FromTagError;
use std::fmt::Display;
use std::ops::RangeTo;

/// This is a custom implementation of nom::recognize_float that does not parse
/// the optional sign before the number, so that expressions like `x+3` parse
/// correctly and not as `x(+3)`.
pub fn recognize_float(i: &str) -> IResult<&str, &str> {
    recognize(
        tuple((
            // We replace the value with () to avoid the types mismatching on
            // the alt branches. We don't need any value here since all this
            // parsing is being done inside a `recognize` parser.
            alt((
                value((), tuple((digit1, opt(pair(char('.'), opt(digit1)))))),
                value((), tuple((char('.'), digit1))),
            )),
            opt(tuple((alt((char('e'), char('E'))), digit1))),
        ))
    )(i)
}

/// This is a custom implementation of `nom::multi::fold_many0` that avoids the
/// `Clone` bound on the initial value for the accumulator. The function returns
/// an `impl FnOnce` type instead of nom's usual `impl Fn` to avoid having to
/// clone `init` despite being moved into the closure.
pub fn fold_many0_once<I, O, E, F, G, R>(f: F, init: R, g: G) -> impl FnOnce(I) -> IResult<I, R, E>
    where
        I: Clone + PartialEq,
        F: Fn(I) -> IResult<I, O, E>,
        G: Fn(R, O) -> R,
        E: ParseError<I>,
{
    move |i: I| {
        let mut res = init;
        let mut input = i.clone();

        loop {
            let i_ = input.clone();
            match f(i_) {
                Ok((i, o)) => {
                    // loop trip must always consume (otherwise infinite loops)
                    if i == input {
                        return Err(Err::Error(E::from_error_kind(input, ErrorKind::Many0)));
                    }

                    res = g(res, o);
                    input = i;
                }
                Err(Err::Error(_)) => {
                    return Ok((input, res));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

fn line_comment<'a, E>(input: &'a str) -> IResult<&str, (), E>
    where E: ParseError<&'a str>,
{
    value((), tuple((
        tag("//"),
        is_not("\r\n")
    )))(input)
}

fn inline_comment<'a, E>(input: &'a str) -> IResult<&str, (), E> where E: ParseError<&'a str> {
    value((), tuple((
        tag("/*"),
        take_until("*/"),
        tag("*/"),
    )))(input)
}

pub fn ws0<'a, E>(input: &'a str) -> IResult<&str, (), E> where E: ParseError<&'a str> {
    value((), tuple((
        multispace0,
        many0(
            tuple((
                alt((
                    line_comment,
                    inline_comment,
                )),
                multispace0,
            ))
        )
    )))(input)
}

pub fn ws1<'a, E>(input: &'a str) -> IResult<&str, (), E> where E: ParseError<&'a str> {
    value((), tuple((
        multispace1,
        many0(tuple((
            alt((
                line_comment,
                inline_comment,
            )),
            multispace0,
        )))
    )))(input)
}


pub fn many0_err<I, O, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, (Vec<O>, E), E>
    where
        I: Clone + PartialEq,
        F: Parser<I, O, E>,
        E: ParseError<I>,
{
    move |mut i: I| {
        let mut acc = std::vec::Vec::with_capacity(4);
        loop {
            match f.parse(i.clone()) {
                Err(Err::Error(err)) => return Ok((i, (acc, err))),
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    if i1 == i {
                        return Err(Err::Error(E::from_error_kind(i, ErrorKind::Many0)));
                    }

                    i = i1;
                    acc.push(o);
                }
            }
        }
    }
}


pub fn tag_custom<T, Input, Error: FromTagError<Input>>(
    tag: T,
) -> impl Fn(Input) -> IResult<Input, Input, Error>
    where
        Input: InputTake + Compare<T>,
        T: InputLength + Clone + Display,
{
    // let tag = tag.to_string();
    move |input: Input| {
        let tag_len = tag.input_len();
        let t = tag.clone();
        let res: IResult<_, _, Error> = match input.compare(t) {
            CompareResult::Ok => Ok(input.take_split(tag_len)),
            _ => { Err(Err::Error(Error::from_tag(input, tag.to_string()))) }
        };
        res
    }
}

pub fn fold_many0_once_err<I, O, E, F, G, R>(f: F, init: R, g: G) -> impl FnOnce(I) -> IResult<I, (R, E), E>
    where
        I: Clone + PartialEq,
        F: Fn(I) -> IResult<I, O, E>,
        G: Fn(R, O) -> R,
        E: ParseError<I>,
{
    move |i: I| {
        let mut res = init;
        let mut input = i.clone();

        loop {
            let i_ = input.clone();
            match f(i_) {
                Ok((i, o)) => {
                    // loop trip must always consume (otherwise infinite loops)
                    if i == input {
                        return Err(Err::Error(E::from_error_kind(input, ErrorKind::Many0)));
                    }

                    res = g(res, o);
                    input = i;
                }
                Err(Err::Error(err)) => return Ok((input, (res, err))),
                Err(e) => return Err(e),
            }
        }
    }
}

pub fn remaining<I, O, F, E>(mut parser: F) -> impl FnMut(I) -> IResult<I, (I, O), E>
    where
        I: Clone + Offset + Slice<RangeTo<usize>>,
        E: ParseError<I>,
        F: Parser<I, O, E>,
{
    move |input: I| {
        let i = input.clone();
        match parser.parse(i) {
            Ok((remaining, result)) => {
                Ok((remaining.clone(), (remaining, result)))
            }
            Err(e) => Err(e),
        }
    }
}
