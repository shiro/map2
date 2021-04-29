use super::*;

pub(super) fn key_mapping_inline(input: &str) -> Res<&str, Expr> {
    context(
        "key_mapping_inline",
        tuple((
            key_action,
            tag("::"),
            alt((
                key_sequence,
                map(key_action, |v| vec![v]),
            ))
        )),
    )(input).and_then(|(next, v)| {
        let (from, to) = (v.0, v.2);

        Ok((next, match from {
            // ParsedKeyAction::KeyAction(from) => {
            //     Expr::map_key_block(from, Block::new()
            //         .tap_mut(|b| b.statements = to.into_iter().map(|v| {
            //             // TODO handle modifiers (just shift) by checking if it's already down
            //             Stmt::Expr(Expr::KeyAction(KeyAction::new(v.key, v.value)))
            //         }).collect()),
            //     )
            // }
            ParsedKeyAction::KeyClickAction(from) => {
                //     match to {
                //         ParsedKeyAction::KeyAction(_) => { unimplemented!() }
                //         ParsedKeyAction::KeyClickAction(to) => {
                //             Expr::map_key_click(from, to)
                //         }
                //     }

                // click to click
                if to.len() == 1 {
                    if let Some(ParsedKeyAction::KeyClickAction(to)) = to.get(0) {
                        return Ok((next, Expr::map_key_click(&from, to)));
                    }
                }

                // click to action


                // click to seq
                let expr = Expr::map_key_click_block(from, Block::new()
                    .tap_mut(|b| b.statements = to
                        .into_iter()
                        .fold(vec![], |mut acc, v| match v {
                            ParsedKeyAction::KeyAction(action) => {
                                if action.modifiers.shift { acc.push(Stmt::Expr(Expr::KeyAction(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_DOWN)))); }
                                acc.push(Stmt::Expr(Expr::KeyAction(KeyAction::new(action.key, action.value))));
                                acc
                            }
                            ParsedKeyAction::KeyClickAction(action) => {
                                if action.modifiers.shift { acc.push(Stmt::Expr(Expr::KeyAction(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_DOWN)))); }
                                acc.push(Stmt::Expr(Expr::KeyAction(KeyAction::new(action.key, TYPE_DOWN))));
                                acc.push(Stmt::Expr(Expr::KeyAction(KeyAction::new(action.key, TYPE_UP))));
                                acc
                            }
                        })
                    ),
                );

                expr
            }
            _ => unimplemented!()
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
            ParsedKeyAction::KeyClickAction(from) => { Expr::map_key_click_block(from, to) }
            ParsedKeyAction::KeyAction(from) => unimplemented!(),
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
