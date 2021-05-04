use itertools::Itertools;
use nom::combinator::{map_res, recognize};
use nom::multi::many1;

use super::*;

pub(super) fn key_sequence(input: &str) -> Res<&str, Vec<ParsedKeyAction>> {
    context(
        "key_sequence",
        tuple((
            tag("\""),
            many1(
                alt((
                    map_res(take(1usize), key_action),
                    map_res(
                        recognize(terminated(take_until("}"), tag("}"))),
                        key_action,
                    ),
                )),
            ),
            tag("\""),
        )),
    )(input).and_then(|(next, val)| Ok((next, val.1.into_iter()
        .map(|v| {
            if !v.0.is_empty() { return Err(make_generic_nom_err()); }
            Ok(v.1)
        })
        .fold_ok(vec![], |mut acc, v| {
            acc.push(v);
            acc
        })?
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_sequence() {
        assert_eq!(key_sequence("\"abc\""), Ok(("", vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_B, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
        ])));

        assert_eq!(key_sequence("\"a{b down}\""), Ok(("", vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods { key: *KEY_B, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }),
        ])));
    }

    #[test]
    fn test_key_sequence_mixed() {
        assert_eq!(key_sequence("\"a{b down}c\""), Ok(("", vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_B, TYPE_DOWN, KeyModifierFlags::new())),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
        ])));

        assert_eq!(key_sequence("\"{shift down}a{shift up}\""), Ok(("", vec![
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_LEFT_SHIFT, TYPE_DOWN, KeyModifierFlags::new())),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_LEFT_SHIFT, TYPE_UP, KeyModifierFlags::new())),
        ])));
    }
}