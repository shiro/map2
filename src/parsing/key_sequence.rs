use itertools::Itertools;
use nom::combinator::{map_res, recognize};
use nom::multi::many1;

use super::*;

pub(super) fn key_sequence(input: &str) -> ResNew<&str, Vec<ParsedKeyAction>> {
    many1(
        alt((
            map_res(
                recognize(tuple((
                    key_flags,
                    tag_custom("{"),
                    terminated(take_until("}"), tag_custom("}"))),
                )),
                |input| {
                    let (input, action) = alt((
                        key_action_with_flags,
                        key_action
                    ))(input)?;
                    // TODO properly propagate child error
                    if !input.is_empty() {
                        return Err(make_generic_nom_err_new(input));
                    }

                    Ok((input, action))
                },
            ),
            map_res(take(1usize), key_action),
        )),
    )(input).and_then(|(next, val)| {
        let seq = val.into_iter()
            .map(|v| {
                if !v.0.is_empty() { return Err(make_generic_nom_err_new(input)); }
                Ok(v.1.0)
            })
            .fold_ok(vec![], |mut acc, v| {
                acc.push(v);
                acc
            })?;
        Ok((next, (seq, None)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_key_sequence() {
    //     assert_eq!(key_sequence("\"abc\""), nom_ok(vec![
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_B, modifiers: KeyModifierFlags::new() }),
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
    //     ]));
    //
    //     assert_eq!(key_sequence("\"a{b down}\""), nom_ok(vec![
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
    //         ParsedKeyAction::KeyAction(KeyActionWithMods { key: *KEY_B, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }),
    //     ]));
    // }
    //
    // #[test]
    // fn test_key_sequence_mixed() {
    //     assert_eq!(key_sequence("\"a{b down}c\""), nom_ok(vec![
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
    //         ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_B, TYPE_DOWN, KeyModifierFlags::new())),
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
    //     ]));
    //
    //     assert_eq!(key_sequence("\"{shift down}a{shift up}\""), nom_ok(vec![
    //         ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_LEFT_SHIFT, TYPE_DOWN, KeyModifierFlags::new())),
    //         ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
    //         ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_LEFT_SHIFT, TYPE_UP, KeyModifierFlags::new())),
    //     ]));
    // }
}