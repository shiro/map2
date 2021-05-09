use super::*;
use nom::error::{ErrorKind, ParseError, FromExternalError};
use nom::{InputLength};
use std::fmt::Debug;


pub(super) type ResNew<I, O> = IResult<I, (O, Option<CustomError<I>>), CustomError<I>>;

pub(super) fn make_generic_nom_err_new<I>(input: I) -> NomErr<CustomError<I>> {
    NomErr::Error(CustomError { input, expected: vec![] })
}

pub(super) fn make_generic_nom_err_options<I>(input: I, options: Vec<String>) -> NomErr<CustomError<I>> {
    NomErr::Error(CustomError { input, expected: options })
}


#[derive(Debug, PartialEq)]
pub(super) struct CustomError<I> {
    pub(super) input: I,
    pub(super) expected: Vec<String>,
}

impl<I> ParseError<I> for CustomError<I> where I: InputLength {
    fn from_error_kind(input: I, _: ErrorKind) -> Self {
        CustomError { input, expected: vec![] }
    }

    fn from_char(input: I, ch: char) -> Self {
        CustomError { input, expected: vec![ch.to_string()] }
    }

    fn or(mut self, mut other: Self) -> Self {
        if other.input.input_len() < self.input.input_len() {
            return other;
        } else if (other.input.input_len() > self.input.input_len()) {
            return self;
        }
        other.expected.append(&mut self.expected);
        other
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self { other }
}

impl<I, E> FromExternalError<I, E> for CustomError<I> {
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        Self { input, expected: vec![] }
    }
}


pub trait FromTagError<I>: Sized {
    fn from_tag(input: I, tag: String) -> Self;
}

impl<Input> FromTagError<Input> for CustomError<Input> {
    fn from_tag(input: Input, tag: String) -> Self {
        Self { input, expected: vec![format!("'{}'", tag)] }
    }
}


pub(super) fn convert_custom_error<I: core::ops::Deref<Target=str>>(
    input: I,
    err: &CustomError<I>,
) -> String {
    use std::fmt::Write;
    use nom::Offset;

    let mut result = String::new();

    let expected;
    if err.expected.is_empty() {
        expected = "valid token (no suggestion)".to_string();
    } else {
        let mut options = err.expected.clone();
        options.sort();
        options.dedup();

        expected = format!("[ {} ]", options.join(", "));
    }

    let substring = &err.input;

    let offset = input.offset(&substring);

    if input.is_empty() {
        // TODO handle EOF
        // match kind {
        //     VerboseErrorKind::Char(c) => {
        //         write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c)
        //     }
        //     VerboseErrorKind::Context(s) => write!(&mut result, "{}: in {}, got empty input\n\n", i, s),
        //     VerboseErrorKind::Nom(e) => write!(&mut result, "{}: in {:?}, got empty input\n\n", i, e),
        // }
    } else {
        let prefix = &input.as_bytes()[..offset];

        // Count the number of newlines in the first `offset` bytes of input
        let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

        // Find the line that includes the subslice:
        // Find the *last* newline before the substring starts
        let line_begin = prefix
            .iter()
            .rev()
            .position(|&b| b == b'\n')
            .map(|pos| offset - pos)
            .unwrap_or(0);

        // Find the full line after that newline
        let line = input[line_begin..]
            .lines()
            .next()
            .unwrap_or(&input[line_begin..])
            .trim_end();

        // The (1-indexed) column number is the offset of our substring into that line
        let column_number: usize = line.offset(&substring) + 1;

        write!(
            &mut result,
            "err: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected {expected}\n\n",
            // i = i,
            line_number = line_number,
            line = line,
            caret = '^',
            column = column_number,
            expected = expected,
        ).unwrap();
    }

    result
}
