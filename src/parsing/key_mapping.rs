use super::*;

pub(super) fn key_mapping_inline(input: &str) -> ResNew<&str, Expr> {
    tuple((
        key_action_with_flags,
        tag("::"),
        alt((
            key_sequence,
            map(key_action_with_flags, |v| (vec![v.0], None)),
        ))
    )
    )(input).and_then(|(next, v)| {
        let (from, mut to) = (v.0.0, v.2.0);

        let expr = match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    // action to click
                    if let ParsedKeyAction::KeyClickAction(to) = to {
                        return Ok((next, (Expr::map_key_action_click(from, to), None)));
                    }
                    // action to action
                    if let ParsedKeyAction::KeyAction(to) = to {
                        return Ok((next, (Expr::map_key_action_action(from, to), None)));
                    }
                }

                // action to seq
                Expr::map_key_block(from, Block::new()
                    .tap_mut(|b| b.statements = to
                        .to_key_actions()
                        .into_iter()
                        .map(|v| Stmt::Expr(Expr::KeyAction(v)))
                        .collect()),
                )
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    // click to click
                    if let Some(ParsedKeyAction::KeyClickAction(to)) = to.get(0) {
                        return Ok((next, (Expr::map_key_click(&from, to), None)));
                    }
                    // click to action
                    if let ParsedKeyAction::KeyAction(to) = to.remove(0) {
                        return Ok((next, (Expr::map_key_click_action(from, to), None)));
                    }
                }

                // click to seq
                Expr::map_key_click_block(from, Block::new()
                    .tap_mut(|b| b.statements = to
                        .to_key_actions()
                        .into_iter()
                        .map(|v| Stmt::Expr(Expr::KeyAction(v)))
                        .collect()),
                )
            }
        };

        Ok((next, (expr, None)))
    })
}

pub(super) fn key_mapping(input: &str) -> ResNew<&str, Expr> {
    tuple((
        key_action_with_flags,
        tag("::"),
        ws0,
        block,
    ))(input).and_then(|(next, v)| {
        let ((from, _), (to, last_err)) = (v.0, v.3);

        let expr = match from {
            ParsedKeyAction::KeyClickAction(from) => { Expr::map_key_click_block(from, to) }
            ParsedKeyAction::KeyAction(from) => { Expr::map_key_block(from, to) }
        };

        Ok((next, (expr, last_err)))
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_mapping_inline() {
        assert_eq!(key_mapping_inline("a::b"), Ok(("", Expr::map_key_click(
            &KeyClickActionWithMods::new(*KEY_A),
            &KeyClickActionWithMods::new(*KEY_B),
        ))));

        assert_eq!(key_mapping_inline("A::b"), Ok(("", Expr::map_key_click(
            &KeyClickActionWithMods::new(*KEY_A).tap_mut(|v| { v.modifiers.shift(); }),
            &KeyClickActionWithMods::new(*KEY_B),
        ))));
    }

    #[test]
    fn test_key_mapping() {
        assert_eq!(key_mapping("a::{}"), Ok(("", Expr::map_key_click_block(
            KeyClickActionWithMods::new(*KEY_A),
            Block::new(),
        ))));
    }

    #[test]
    fn test_key_sequence() {
        assert_eq!(key_mapping_inline("a::\"ab\""), Ok(("", Expr::KeyMapping(vec![
            KeyMapping {
                from: KeyActionWithMods::new(*KEY_A, TYPE_DOWN, KeyModifierFlags::new()),
                to: Block::new().tap_mut(|b| {
                    b.statements = vec![
                        Stmt::Expr(Expr::KeyAction(KeyAction::new(*KEY_A, TYPE_DOWN))),
                        Stmt::Expr(Expr::KeyAction(KeyAction::new(*KEY_A, TYPE_UP))),
                        Stmt::Expr(Expr::KeyAction(KeyAction::new(*KEY_B, TYPE_DOWN))),
                        Stmt::Expr(Expr::KeyAction(KeyAction::new(*KEY_B, TYPE_UP))),
                    ];
                }),
            },
            KeyMapping { from: KeyActionWithMods::new(*KEY_A, TYPE_REPEAT, KeyModifierFlags::new()), to: Block::new() },
            KeyMapping { from: KeyActionWithMods::new(*KEY_A, TYPE_UP, KeyModifierFlags::new()), to: Block::new() },
        ]))));
    }

    #[test]
    fn test_key_mapping_complex() {
        // TODO add when implemented
        // assert_eq!(key_mapping("{a down}::{}"), Ok(("", Expr::map_key_click_block(
        //     KeyClickActionWithMods::new(*KEY_A),
        //     Block::new(),
        // ))));
    }
}
