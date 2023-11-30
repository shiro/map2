use itertools::Itertools;

use crate::xkb::XKBTransformer;

use super::*;

pub fn key_sequence(input: &str) -> ParseResult<&str, Vec<ParsedKeyAction>> {
    key_sequence_utf(None)(input)
}

pub fn key_sequence_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ParseResult<&'a str, Vec<ParsedKeyAction>> + 'a {
    move |input: &str| {
        many1(key_action_utf(transformer))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_input() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();
        let key_sequence = key_sequence_utf(Some(&t));

        assert_eq!(key_sequence("abc"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_A.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_B.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_C.into(), modifiers: KeyModifierFlags::new() }),
        ]));

        assert_eq!(key_sequence("Hi there!"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods {
                key: KEY_H.into(),
                modifiers: KeyModifierFlags::new().tap_mut(|x| x.shift()),
            }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_I.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_SPACE.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_T.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_H.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_E.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_R.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_E.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods {
                key: KEY_1.into(),
                modifiers: KeyModifierFlags::new().tap_mut(|x| x.shift()),
            }),
        ]));

        assert_eq!(key_sequence("a{b down}"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: KEY_A.into(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods { key: KEY_B.into(), value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }),
        ]));
    }

    #[test]
    fn sequence_mixed() {
        assert_eq!(key_sequence("a{b down}c"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: Key::from_str("a").unwrap(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(Key::from_str("b").unwrap(), TYPE_DOWN, KeyModifierFlags::new())),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: Key::from_str("c").unwrap(), modifiers: KeyModifierFlags::new() }),
        ]));

        assert_eq!(key_sequence("{shift down}a{shift up}"), nom_ok(vec![
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(KEY_LEFTSHIFT.into(), TYPE_DOWN, KeyModifierFlags::new())),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: Key::from_str("a").unwrap(), modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods::new(KEY_LEFTSHIFT.into(), TYPE_UP, KeyModifierFlags::new())),
        ]));
    }

    #[test]
    fn sequence_special_chars() {
        assert_eq!(key_sequence("{relative X 55}"), nom_ok(vec![
            ParsedKeyAction::Action(KeyAction::new(Key { event_code: EventCode::EV_REL(REL_X) }, 55)),
        ]));
    }

    #[test]
    fn sequence_escaped_special_chars() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_eq!(key_sequence_utf(Some(&t))("\\{"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(
                KEY_LEFTBRACE.into(), KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )),
        ]));

        assert_eq!(key_sequence_utf(Some(&t))("\\}"), nom_ok(vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(
                KEY_RIGHTBRACE.into(), KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )),
        ]));
    }

    #[test]
    fn sequence_invalid_multiple_keys_in_special_group() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_nom_err(key_sequence_utf(Some(&t))("{abc}"), "{abc}");
    }
}