use anyhow::*;
use futures::StreamExt;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, opt};
use nom::Err as NomErr;
use nom::error::{context, VerboseError};
use nom::IResult;
use nom::multi::{many0, many1};
use nom::sequence::*;
use tap::Tap;

use crate::*;
use crate::block_ext::ExprVecExt;
use evdev_rs::enums::EventType;

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
            _ => unreachable!(),
        })
    )
}

fn variable_assignment(input: &str) -> Res<&str, Expr> {
    context(
        "variable_declaration",
        tuple((
            tag("let"),
            multispace0,
            variable_name,
            multispace0,
            tag("="),
            multispace0,
            boolean,
        )),
    )(input).map(|(next, parts)|
        (next, Expr::Assign(parts.2, Box::new(parts.6)))
    )
}

fn key_flags(input: &str) -> Res<&str, KeyModifierFlags> {
    context("key_flags", many0(one_of("^!+#")))(input).and_then(|(next, val)| {
        let mut flags = KeyModifierFlags::new();
        for v in val {
            match v {
                '!' => { if !flags.alt { flags.alt(); } else { return Err(make_generic_nom_err()); } }
                '^' => { if !flags.ctrl { flags.ctrl(); } else { return Err(make_generic_nom_err()); } }
                '+' => { if !flags.shift { flags.shift(); } else { return Err(make_generic_nom_err()); } }
                '#' => { if !flags.meta { flags.meta(); } else { return Err(make_generic_nom_err()); } }
                _ => unreachable!()
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
            let key_name = "KEY_".to_string()
                .tap_mut(|s| s.push_str(&val.to_uppercase()));

            let key = Key::from_str(&EventType::EV_KEY, key_name.as_str())
                .map_err(|_| make_generic_nom_err())?;

            // only 1 char and it's uppercase
            let mut it = val.chars();
            if it.next().unwrap().is_uppercase() && it.next().is_none() {
                return Ok((next, ParsedSingleKey::CapitalKey(key.clone())));
            }

            Ok((next, ParsedSingleKey::Key(key.clone())))
        })
}

#[derive(PartialEq, Debug, Clone)]
enum ParsedKeyAction {
    KeyAction(KeyAction),
    KeyClickAction(KeyClickActionWithMods),
    KeySequence(Vec<Expr>),
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


fn key_sequence(input: &str) -> Res<&str, Vec<Expr>> {
    context(
        "key_sequence",
        tuple((
            tag("\""),
            take_until("\""),
            tag("\""),
        )),
    )(input).and_then(|(next, v)| {
        Ok((next, vec![].append_string_sequence(v.1)))
    })
}

fn key_mapping_inline(input: &str) -> Res<&str, Expr> {
    context(
        "key_mapping_inline",
        tuple((
            key_action,
            tag("::"),
            alt((
                map(key_sequence, |seq| ParsedKeyAction::KeySequence(seq)),
                key_action,
            ))
        )),
    )(input).and_then(|(next, v)| {
        let (from, to) = (v.0, v.2);

        Ok((next, match from {
            ParsedKeyAction::KeyAction(_) => { unimplemented!() }
            ParsedKeyAction::KeyClickAction(from) => {
                match to {
                    ParsedKeyAction::KeyAction(_) => { unimplemented!() }
                    ParsedKeyAction::KeyClickAction(to) => {
                        Expr::map_key_click(from, to)
                    }
                    ParsedKeyAction::KeySequence(expr) => {
                        Expr::map_key_block(from, Block::new()
                            .tap_mut(|b| b.statements = expr.into_iter().map(Stmt::Expr).collect()),
                        )
                    }
                }
            }
            ParsedKeyAction::KeySequence(_) => return Err(make_generic_nom_err())
        }))
    })
}

fn expr(input: &str) -> Res<&str, Expr> {
    context(
        "expr",
        tuple((
            alt((
                variable_assignment,
                key_mapping_inline,
            )),
            multispace0,
        )),
    )(input).map(|(next, v)| (next, v.0))
}

fn stmt(input: &str) -> Res<&str, Stmt> {
    context(
        "stmt",
        tuple((
            alt((
                map(expr, Stmt::Expr),
                map(block, Stmt::Block),
            )),
            tag(";"),
        )),
    )(input).map(|(next, val)| (next, val.0))
}

fn block_body(input: &str) -> Res<&str, Block> {
    context(
        "block_body",
        opt(tuple((
            stmt,
            many0(tuple((
                multispace0,
                stmt,
            ))),
        ))),
    )(input).map(|(next, v)| {
        match v {
            Some((s1, s2)) => {
                (next, Block::new().tap_mut(|b| {
                    let mut statements: Vec<Stmt> = s2.into_iter().map(|x| x.1).collect();
                    statements.insert(0, s1);
                    b.statements = statements;
                }))
            }
            _ => (next, Block::new())
        }
    })
}

fn block(input: &str) -> Res<&str, Block> {
    context(
        "block",
        tuple((
            tag("{"),
            multispace0,
            block_body,
            multispace0,
            tag("}")
        )),
    )(input).map(|(next, v)| (next, v.2))
}

pub(crate) fn parse_script<>(raw_script: &str) -> Result<Block> {
    match block_body(raw_script) {
        Ok(v) => Ok(v.1),
        Err(_) => Err(anyhow!("parsing failed"))
    }
}

#[cfg(test)]
mod tests {
    use input_linux_sys::EV_KEY;
    use nom::error::{ErrorKind, VerboseErrorKind};
    use tap::Tap;

    use super::*;

    #[test]
    fn test_value() {
        assert!(matches!(boolean("true"), Ok(("", Expr::Boolean(true)))));
        assert!(matches!(boolean("false"), Ok(("", Expr::Boolean(false)))));
        assert!(matches!(boolean("foo"), Err(..)));
    }

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
        assert_eq!(key("a"), Ok(("", ParsedSingleKey::Key(*KEY_A))));
        // assert_eq!(key("mouse5"), Ok(("", ParsedSingleKey::Key(*KEY_MOUSE5))));
        assert_eq!(key("A"), Ok(("", ParsedSingleKey::CapitalKey(*KEY_A))));
        assert_eq!(key("enter"), Ok(("", ParsedSingleKey::Key(*KEY_ENTER))));
        assert!(matches!(key("entert"), Err(..)));
    }

    #[test]
    fn test_key_sequence() {
        assert_eq!(key_sequence("\"hello world\""), Ok(("", vec![]
            .append_string_sequence("hello world")
        )));
    }

    #[test]
    fn test_key_action() {
        assert_eq!(key_action("!a"), Ok(("", ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(
                *KEY_A,
                *KeyModifierFlags::new().alt(),
            )))));

        assert_eq!(key_action("!#^a"), Ok(("", ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(
                *KEY_A,
                *KeyModifierFlags::new().ctrl().alt().meta(),
            )))));

        assert_eq!(key_action("A"), Ok(("", ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(
                *KEY_A,
                *KeyModifierFlags::new().shift(),
            )))));

        assert_eq!(key_action("+A"), Ok(("", ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(
                *KEY_A,
                *KeyModifierFlags::new().shift(),
            )))));

        assert!(matches!(key_action("+al"), Err(..)));

        assert!(matches!(key_action("++a"), Err(..)));
    }

    #[test]
    fn test_key_mapping_inline() {
        assert_eq!(key_mapping_inline("a::b"), Ok(("", Expr::map_key_click(
            KeyClickActionWithMods::new(*KEY_A),
            KeyClickActionWithMods::new(*KEY_B),
        ))));

        assert_eq!(key_mapping_inline("A::b"), Ok(("", Expr::map_key_click(
            KeyClickActionWithMods::new(*KEY_A).tap_mut(|v| { v.modifiers.shift(); }),
            KeyClickActionWithMods::new(*KEY_B),
        ))));
    }

    #[test]
    fn test_block() {
        assert_eq!(block_body("a::b;"), Ok(("", Block::new()
            .tap_mut(|b| { b.statements.push(stmt("a::b;").unwrap().1); })
        )));

        assert_eq!(block("{ let foo = true; }"), Ok(("", Block::new()
            .tap_mut(|b| { b.statements.push(stmt("let foo = true;").unwrap().1); })
        )));

        assert_eq!(block("{ let foo = true; a::b; b::c; }"), Ok(("", Block::new()
            .tap_mut(|b| {
                b.statements.push(stmt("let foo = true;").unwrap().1);
                b.statements.push(stmt("a::b;").unwrap().1);
                b.statements.push(stmt("b::c;").unwrap().1);
            })
        )));
    }

    #[test]
    fn test_assignment() {
        assert_eq!(variable_name("hello2"), Ok(("", "hello2".to_string())));
        assert_eq!(variable_assignment("let foo = true"),
                   Ok(("", Expr::Assign("foo".to_string(), Box::new(boolean("true").unwrap().1))))
        );

        assert!(matches!(variable_name("2hello"), Err(..)));
    }
}