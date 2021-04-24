use super::*;

pub(super) fn key_mapping_inline(input: &str) -> Res<&str, Expr> {
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

pub(super) fn key_mapping(input: &str) -> Res<&str, Expr> {
    context(
        "key_mapping",
        tuple((
            key_action,
            tag("::"),
            multispace0,
            block,
        )),
    )(input).and_then(|(next, v)| {
        let (from, to) = (v.0, v.3);

        let expr = match from {
            ParsedKeyAction::KeyClickAction(from) => { Expr::map_key_block(from, to) }
            ParsedKeyAction::KeyAction(from) => unimplemented!(),
            ParsedKeyAction::KeySequence(from) => return Err(make_generic_nom_err()),
        };
        Ok((next, expr))
    })
}


#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_key_mapping() {
        assert_eq!(key_mapping("a::{}"), Ok(("", Expr::map_key_block(
            KeyClickActionWithMods::new(*KEY_A),
            Block::new(),
        ))));
    }
}
