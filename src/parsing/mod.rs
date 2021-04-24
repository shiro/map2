use anyhow::*;
use evdev_rs::enums::EventType;
use futures::StreamExt;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{map, opt};
use nom::Err as NomErr;
use nom::error::{context, VerboseError};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::*;
use tap::Tap;

use assignment::*;
use identifier::*;
use lambda::*;

use crate::*;
use crate::block_ext::ExprVecExt;
use crate::parsing::custom_combinators::fold_many0_once;
use crate::parsing::identifier::ident;

mod custom_combinators;
mod identifier;
mod lambda;
mod assignment;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn make_generic_nom_err<'a>() -> NomErr<VerboseError<&'a str>> { NomErr::Error(VerboseError { errors: vec![] }) }

static STACK_SIZE: usize = 255;

fn string(input: &str) -> Res<&str, Expr> {
    context(
        "string",
        tuple((tag("\""), take_until("\""), tag("\""))),
    )(input)
        .map(|(next, v)| (next, Expr::String(v.1.to_string())))
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


fn expr_simple(input: &str) -> Res<&str, Expr> {
    context(
        "expr_simple",
        tuple((
            alt((
                boolean,
                string,
                lambda,
                variable_assignment,
                function_call,
                key_mapping_inline,
            )),
            multispace0,
        )),
    )(input).map(|(next, v)| (next, v.0))
}

fn expr(i: &str) -> Res<&str, Expr> {
    let (i, init) = expr_simple(i)?;
    fold_many0_once(
        |i: &str| {
            context(
                "expr",
                tuple((
                    multispace0,
                    alt((tag("=="), tag("!="))),
                    multispace0,
                    expr_simple,
                )),
            )(i)
        },
        init,
        |acc, (_, op, _, val)| {
            match op {
                "==" => Expr::Eq(Box::new(acc), Box::new(val)),
                // TODO implement neq
                "!=" => Expr::Eq(Box::new(acc), Box::new(val)),
                _ => unreachable!()
            }
        },
    )(i)
}

fn if_stmt(input: &str) -> Res<&str, Stmt> {
    context(
        "if_stmt",
        tuple((
            tag("if"),
            multispace0,
            tag("("),
            multispace0,
            expr,
            multispace0,
            tag(")"),
            multispace0,
            block,
        )),
    )(input).map(|(next, v)| (next, Stmt::If(v.4, v.8)))
}


fn function_arg(input: &str) -> Res<&str, Expr> {
    context("function_arg", expr)(input)
}

fn function_call(input: &str) -> Res<&str, Expr> {
    context(
        "function_call",
        tuple((
            ident,
            tag("("),
            multispace0,
            opt(tuple((
                function_arg,
                multispace0,
                many0(tuple((
                    tag(","),
                    multispace0,
                    function_arg,
                    multispace0,
                ))),
            ))),
            tag(")"),
        )),
    )(input).map(|(next, v)| {
        match v.3 {
            Some(arg_v) => {
                let mut args: Vec<Expr> = arg_v.2.into_iter().map(|x| x.2).collect();
                args.insert(0, arg_v.0);
                (next, Expr::FunctionCall(v.0, args))
            }
            _ => (next, Expr::FunctionCall(v.0, vec![]))
        }
    })
}

fn stmt(input: &str) -> Res<&str, Stmt> {
    context(
        "stmt",
        tuple((
            alt((
                if_stmt,
                map(tuple((expr, tag(";"))), |v| Stmt::Expr(v.0)),
                map(block, Stmt::Block),
            )),
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
    use nom::error::{ErrorKind, VerboseErrorKind};
    use tap::Tap;

    use crate::block_ext::ExprVecExt;

    use super::*;

    #[test]
    fn test_function_call() {
        assert_eq!(function_call("foobar()"), Ok(("", Expr::FunctionCall("foobar".to_string(), vec![]))));
        assert_eq!(function_call("foobar(\"hello\", true)"), Ok(("", Expr::FunctionCall("foobar".to_string(), vec![
            Expr::String("hello".to_string()),
            Expr::Boolean(true),
        ]))));
        assert_eq!(function_call("foobar(true == true)"), Ok(("", Expr::FunctionCall("foobar".to_string(), vec![
            expr("true == true").unwrap().1
        ]))));
    }

    #[test]
    fn test_if_stmt() {
        assert_eq!(if_stmt("if(true){ a::b; }"), Ok(("", Stmt::If(
            expr("true").unwrap().1,
            block("{a::b;}").unwrap().1,
        ))));
        assert_eq!(stmt("if(true){ a::b; }"), Ok(("", Stmt::If(
            expr("true").unwrap().1,
            block("{a::b;}").unwrap().1,
        ))));

        assert_eq!(stmt("if(\"a\" == \"a\"){ a::b; }"), Ok(("", Stmt::If(
            expr("\"a\" == \"a\"").unwrap().1,
            block("{a::b;}").unwrap().1,
        ))));
        assert_eq!(stmt("if(foo() == \"a\"){ a::b; }"), Ok(("", Stmt::If(
            Expr::Eq(
                Box::new(Expr::FunctionCall("foo".to_string(), vec![])),
                Box::new(Expr::String("a".to_string())),
            ),
            block("{a::b;}").unwrap().1,
        ))));
    }

    #[test]
    fn test_value() {
        assert!(matches!(boolean("true"), Ok(("", Expr::Boolean(true)))));
        assert!(matches!(boolean("false"), Ok(("", Expr::Boolean(false)))));
        assert!(matches!(boolean("foo"), Err(..)));

        assert_eq!(string("\"hello world\""), Ok(("", Expr::String("hello world".to_string()))));
    }

    #[test]
    fn test_operator_equal() {
        assert_eq!(expr("true == true"), Ok(("", Expr::Eq(
            Box::new(Expr::Boolean(true)),
            Box::new(Expr::Boolean(true)),
        ))));
        assert_eq!(expr("\"hello world\" == \"hello world\""), Ok(("", Expr::Eq(
            Box::new(Expr::String("hello world".to_string())),
            Box::new(Expr::String("hello world".to_string())),
        ))));
        assert_eq!(expr("\"22hello\" == true"), Ok(("", Expr::Eq(
            Box::new(Expr::String("22hello".to_string())),
            Box::new(Expr::Boolean(true)),
        ))));
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

        assert_eq!(block_body("if(true){a::b;}"), Ok(("", Block::new().tap_mut(|b| {
            b.statements = vec![
                if_stmt("if(true){a::b;}").unwrap().1
            ];
        }))));
    }
}