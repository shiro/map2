use nom::branch::alt;
use nom::character::complete::{char, digit1};
use nom::combinator::{opt, recognize, value};
use nom::Err;
use nom::error::{ErrorKind, ParseError};
use nom::IResult;
use nom::sequence::{pair, tuple};

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
/// `Clone` bound on the initial value for the accumlator. The function returns
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
