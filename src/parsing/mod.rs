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

use expression::*;
use function::*;
use identifier::*;
use if_statement::*;
use key::*;
use key_action::*;
use key_mapping::*;
use key_sequence::*;
use lambda::*;
use primitives::*;
use variable::*;
use for_loop::*;
use return_statement::*;
use continue_statement::*;
use custom_combinators::*;
use error::*;

use crate::*;
use crate::parsing::identifier::ident;

pub mod parser;
mod return_statement;
mod continue_statement;
mod custom_combinators;
mod expression;
mod function;
mod identifier;
mod if_statement;
mod key;
mod key_action;
mod key_mapping;
mod key_sequence;
mod lambda;
mod primitives;
mod variable;
mod for_loop;
mod error;


fn stmt(input: &str) -> ResNew<&str, Stmt> {
    tuple((
              alt((
                  return_statement,
                  continue_statement,
                  // if_stmt,
                  // for_loop,
                  map(tuple((expr, tag(";"))), |v| Stmt::Expr(v.0)),
                  // map(block, Stmt::Block),
              )),
          ),
    )(input)
        .map_err(|v| NomErr::Error(CustomError { input, expected: vec!["statement".to_string()] }))
        .map(|(next, val)| (next, (val.0, None)))
}

fn block_body(input: &str) -> ResNew<&str, Block> {
    // opt(tuple((
    //     stmt,
    many0_err(tuple((
        ws0,
        stmt,
    ))
              // ))),
    )(input).map(|(next, v)| {
        // match v {
        //     Some((s1, (s2, last_err))) => {
        let (s2, last_err) = v;
        (next, (
            Block::new().tap_mut(|b| {
                let mut statements: Vec<Stmt> = s2.into_iter().map(|x| x.1.0).collect();
                // statements.insert(0, s1);
                b.statements = statements;
            }),
            Some(last_err)
        ))
        // }
        // _ => (next, (Block::new()))
        // }
    })
}

fn block(input: &str) -> ResNew<&str, Block> {
    tuple((
              tag("{"),
              ws0,
              block_body,
              ws0,
              tag("}")
          ),
    )(input)
        .map(|(next, v)| (next, v.2))
}

fn global_block(input: &str) -> ResNew<&str, Block> {
    tuple((ws0, block_body, ws0),
    )(input)
        .map(|(next, v)| (next, v.1))
}


#[cfg(test)]
mod tests {
    use nom::error::{ErrorKind, VerboseErrorKind};
    use tap::Tap;

    use crate::block_ext::ExprVecExt;

    use super::*;

    #[test]
    fn test_key() {
        assert_eq!(key("a"), Ok(("", ParsedSingleKey::Key(*KEY_A))));
        // assert_eq!(key("mouse5"), Ok(("", ParsedSingleKey::Key(*KEY_MOUSE5))));
        assert_eq!(key("A"), Ok(("", ParsedSingleKey::CapitalKey(*KEY_A))));
        assert_eq!(key("enter"), Ok(("", ParsedSingleKey::Key(*KEY_ENTER))));
        assert!(matches!(key("entert"), Err(..)));
    }

    #[test]
    fn test_key_action() {
        assert_eq!(key_action_with_flags("!a"), Ok(("", ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(
                *KEY_A,
                KeyModifierFlags::new().tap_mut(|v| v.alt()),
            )))));

        // assert_eq!(key_action("!#^a"), Ok(("", ParsedKeyAction::KeyClickAction(
        //     KeyClickActionWithMods::new_with_mods(
        //         *KEY_A,
        //         *KeyModifierFlags::new().ctrl().alt().meta(),
        //     )))));
        //
        // assert_eq!(key_action("A"), Ok(("", ParsedKeyAction::KeyClickAction(
        //     KeyClickActionWithMods::new_with_mods(
        //         *KEY_A,
        //         *KeyModifierFlags::new().shift(),
        //     )))));
        //
        // assert_eq!(key_action("+A"), Ok(("", ParsedKeyAction::KeyClickAction(
        //     KeyClickActionWithMods::new_with_mods(
        //         *KEY_A,
        //         *KeyModifierFlags::new().shift(),
        //     )))));
        //
        // assert!(matches!(key_action("+al"), Err(..)));
        //
        // assert!(matches!(key_action("++a"), Err(..)));
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