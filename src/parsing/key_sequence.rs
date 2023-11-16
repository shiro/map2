use itertools::Itertools;

use crate::xkb::UTFToRawInputTransformer;

use super::*;

pub fn key_sequence(input: &str) -> ResNew2<&str, Vec<ParsedKeyAction>> {
    key_sequence_utf(None)(input)
}

pub fn key_sequence_utf<'a>(
    transformer: Option<&'a UTFToRawInputTransformer>
) -> impl Fn(&'a str) -> ResNew2<&'a str, Vec<ParsedKeyAction>> + 'a {
    move |input: &str| {
        many1(key_action_utf(transformer))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_sequence() {
        assert_eq!(key_sequence("abc"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_B, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
        ]));

        assert_eq!(key_sequence("a{b down}"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods { key: *KEY_B, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }),
        ]));
    }

    #[test]
    fn test_key_sequence_mixed() {
        assert_eq!(key_sequence("a{b down}c"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_B, TYPE_DOWN, KeyModifierFlags::new())),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
        ]));

        assert_eq!(key_sequence("{shift down}a{shift up}"), nom_ok(vec![
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_LEFT_SHIFT, TYPE_DOWN, KeyModifierFlags::new())),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(*KEY_LEFT_SHIFT, TYPE_UP, KeyModifierFlags::new())),
        ]));
    }

    #[test]
    fn sequence_escaped_special_chars() {
        let t = UTFToRawInputTransformer::new(None, Some("us"), None, None);

        assert_eq!(key_sequence_utf(Some(&t))("\\{"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(
                *KEY_LEFTBRACE, KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )),
        ]));

        assert_eq!(key_sequence_utf(Some(&t))("\\}"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(
                *KEY_RIGHTBRACE, KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )),
        ]));
    }

    #[test]
    fn sequence_invalid_multiple_keys_in_special_group() {
        let t = UTFToRawInputTransformer::new(None, Some("us"), None, None);

        assert_nom_err(key_sequence_utf(Some(&t))("{abc}"), "{abc}");
    }
}