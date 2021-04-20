use futures::StreamExt;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::opt;
use nom::Err as NomErr;
use nom::error::{context, ErrorKind, VerboseError};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::*;

use crate::*;
use anyhow::Error;
use crate::block_ext::ExprVecExt;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn make_generic_nom_err<'a>() -> NomErr<VerboseError<&'a str>> { NomErr::Error(VerboseError { errors: vec![] }) }


fn variable_name(input: &str) -> Res<&str, String> {
    context(
        "variable name",
        tuple((alpha1, alphanumeric0)),
    )(input)
        .map(|(a, b)| (a, [b.0, b.1].join("")))
}

fn boolean(input: &str) -> Res<&str, Expr> {
    context(
        "value",
        alt((tag("true"), tag("false"))),
    )(input).map(|(next, v)|
        (next, match v {
            "true" => Expr::Boolean(true),
            "false" => Expr::Boolean(false),
            _ => panic!(),
        })
    )
}

fn variable_declaration(input: &str) -> Res<&str, Expr> {
    context(
        "variable_declaration",
        tuple((
            tag("let"),
            multispace1,
            variable_name,
            multispace1,
            tag("="),
            multispace1,
            boolean,
            tag(";")
        )),
    )(input).map(|(next, parts)|
        (next, Expr::Assign(parts.2, Box::new(parts.6)))
    )
}

fn expr(input: &str) -> Res<&str, Expr> {
    context("expr", variable_declaration)(input)
}

fn key_flags(input: &str) -> Res<&str, KeyModifierFlags> {
    context("key_flags", many0(one_of("^!+#")))(input).and_then(|(next, val)| {
        let mut flags = KeyModifierFlags::new();
        for v in val {
            match v {
                ('!') => { if !flags.alt { flags.alt(); } else { return Err(make_generic_nom_err()); } }
                ('^') => { if !flags.ctrl { flags.ctrl(); } else { return Err(make_generic_nom_err()); } }
                ('+') => { if !flags.shift { flags.shift(); } else { return Err(make_generic_nom_err()); } }
                ('#') => { if !flags.meta { flags.meta(); } else { return Err(make_generic_nom_err()); } }
                (_) => unreachable!()
            }
        };
        Ok((next, flags))
    })
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum ParsedSingleKey {
    Key(Key),
    CapitalKey(Key),
}

fn key(input: &str) -> Res<&str, ParsedSingleKey> {
    context("key", alphanumeric1)(input)
        .and_then(|(next, val)| {
            let key = KEY_LOOKUP.get(val.to_lowercase().as_str())
                .ok_or(make_generic_nom_err())?;

            // only 1 char and it's uppercase
            let mut it = val.chars();
            if it.next().unwrap().is_uppercase() && it.next().is_none() {
                return Ok((next, ParsedSingleKey::CapitalKey(key.clone())));
            }

            Ok((next, ParsedSingleKey::Key(key.clone())))
        })
}

// #[derive(Eq, PartialEq, Debug, Clone)]
enum ParsedKeyAction {
    KeyAction(KeyAction),
    KeyClickAction(KeyClickActionWithMods),
}

fn key_action(input: &str) -> Res<&str, ParsedKeyAction> {
    context(
        "key_action",
        tuple((
            key_flags,
            key,
        )),
    )(input).and_then(|(next, parts)| {
        let mut mods = parts.0;
        let key;
        match parts.1 {
            ParsedSingleKey::Key(k) => { key = k; }
            ParsedSingleKey::CapitalKey(k) => {
                mods.shift();
                key = k;
            }
        }

        let action = KeyClickActionWithMods::new_with_mods(key, mods);
        Ok((next, ParsedKeyAction::KeyClickAction(action)))
    })
}


fn key_mapping_inline(input: &str) -> Res<&str, Vec<Expr>> {
    context(
        "key_mapping_inline",
        tuple((
            key_action,
            tag("::"),
            key_action,
            tag(";"),
        )),
    )(input).and_then(|(next, v)| {
        let mut vec = vec![];

        match v.0 {
            ParsedKeyAction::KeyAction(_) => { unimplemented!() }
            ParsedKeyAction::KeyClickAction(from) => {
                match v.2 {
                    ParsedKeyAction::KeyAction(_) => { unimplemented!() }
                    ParsedKeyAction::KeyClickAction(to) => {
                        vec.map_key_click(from, to);
                    }
                }
            }
        }

        Ok((next, vec))
    })
}

fn block(input: &str) -> Res<&str, Block> {
    context(
        "block",
        tuple((
            tag("{"),
            multispace0,
            many0(expr),
            multispace0,
            tag("}")
        )),
    )(input)
        .map(|(next, v)| (next, Block::new().extend_with(v.2)))
}

// struct Foo {
//     a: Mutex<usize>,
// }
//
// impl<T: PartialEq> PartialEq for Foo {
//     fn eq(&self, other: &Self) -> bool {
//         if ::core::ptr::eq(&self, &other) {
//             true
//         } else {
//             other.lock().unwrap().deref() == self.lock().unwrap().deref()
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use input_linux_sys::EV_KEY;
    use nom::error::{ErrorKind, VerboseErrorKind};

    use super::*;

    #[test]
    fn test_key_flags() {
        assert_eq!(key_flags("!"), Ok(("", *KeyModifierFlags::new().alt())));
        assert_eq!(key_flags("^"), Ok(("", *KeyModifierFlags::new().ctrl())));
        assert_eq!(key_flags("+"), Ok(("", *KeyModifierFlags::new().shift())));
        assert_eq!(key_flags("#"), Ok(("", *KeyModifierFlags::new().meta())));

        assert_eq!(key_flags("!#"), Ok(("", *KeyModifierFlags::new().alt().meta())));
        assert_eq!(key_flags("#!"), Ok(("", *KeyModifierFlags::new().alt().meta())));
        assert_eq!(key_flags("#a!"), Ok(("a!", *KeyModifierFlags::new().meta())));
    }

    #[test]
    fn test_key() {
        assert_eq!(key("a"), Ok(("", ParsedSingleKey::Key(KEY_A))));
        assert_eq!(key("A"), Ok(("", ParsedSingleKey::CapitalKey(KEY_A))));
        assert_eq!(key("enter"), Ok(("", ParsedSingleKey::Key(KEY_ENTER))));
        assert!(matches!(key("entert"), Err(..)));
    }

    #[test]
    fn test_value() {
        assert!(matches!(boolean("true"), Ok(("", Expr::Boolean(true)))));
        assert!(matches!(boolean("false"), Ok(("", Expr::Boolean(false)))));
        assert!(matches!(boolean("foo"), Err(..)));
    }

    #[test]
    fn test_block() {
        assert!(matches!(block("{ let foo = true; }"), Ok(("", ..))));
    }

    #[test]
    fn test_assignment() {
        assert_eq!(variable_name("hello2"), Ok(("", "hello2".to_string())));
        assert!(matches!(variable_name("2hello"), Err(..)));

        // assert!(matches!(variable_declaration("let hello2 = true;"), Ok(("", Expr::Assign("hello2".to_string(), Box::new(Expr::Boolean(true)))))));
    }

    // #[test]
    // fn test_scheme() {
    //     assert_eq!(scheme("https://yay"), Ok(("yay", "https://")));
    //     assert_eq!(scheme("http://yay"), Ok(("yay", "http://")));
    //     assert!(matches!( scheme("bla://yay"), Err(NomErr::Error( .. ))));
    // }
}