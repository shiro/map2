use std::fmt::Display;

use nom::{Compare, CompareResult, Err, InputLength, InputTake};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::value;
use nom::error::{ ParseError};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::tuple;

use crate::parsing::error::FromTagError;

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
