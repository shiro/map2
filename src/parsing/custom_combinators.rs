use std::fmt::Display;

use nom::{Compare, CompareResult, Err, InputLength, InputTake};
use nom::IResult;
use nom::sequence::Tuple;

use crate::parsing::error::FromTagError;

use super::*;

pub fn tuple<I: Clone, O, List: Tuple<I, O, CustomError<I>>>(
    mut l: List,
) -> impl FnMut(I) -> ParseResult<I, O> {
    move |i: I| {
        let res = l.parse(i.clone());
        if res.is_err() {
            return Err(make_generic_nom_err_new(i));
        }
        res
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

pub fn tag_custom_no_case<T, Input, Error: FromTagError<Input>>(
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

        let res: IResult<_, _, Error> = match input.compare_no_case(t) {
            CompareResult::Ok => Ok(input.take_split(tag_len)),
            _ => { Err(Err::Error(Error::from_tag(input, tag.to_string()))) }
        };
        res
    }
}


pub fn surrounded_group<'a, Output>(
    from_token: &'a str,
    to_token: &'a str,
    mut parser: impl FnMut(&'a str) -> ParseResult<&'a str, Output> + 'a,
) -> Box<dyn FnMut(&'a str) -> ParseResult<&'a str, Output> + 'a> {
    Box::new(move |input| {
        map_res(
            tuple((
                tag_custom(from_token),
                terminated(take_until(to_token), tag_custom(to_token))
            )),
            |(_, input)| {
                let (input, res) = parser(input)?;
                if !input.is_empty() { return Err(make_generic_nom_err_new(input)); }
                Ok(res)
            },
        )(input)
    })
}