use itertools::Itertools;

use crate::xkb::XKBTransformer;

use super::*;

pub fn key_sequence(input: &str) -> ParseResult<&str, (Vec<ParsedKeyAction>, CustomError<&str>)> {
    key_sequence_utf(None)(input)
}

pub fn key_sequence_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ParseResult<&'a str, (Vec<ParsedKeyAction>, CustomError<&'a str>)> + 'a {
    move |input: &str| {
        many1_with_last_err(key_action_utf(transformer))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_input() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();
        let key_sequence = key_sequence_utf(Some(&t));

        assert_eq!(key_sequence("abc"), nom_ok((
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_A.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_B.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_C.into())),
            ],
            CustomError { input: "", expected: vec![] }
        )));

        assert_eq!(key_sequence("aa{"), Ok(("{", (
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_A.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_A.into())),
            ],
            CustomError { input: "{", expected: vec![] }
        ))));

        assert_eq!(key_sequence("Hi there!"), nom_ok((
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods {
                    key: KEY_H.into(),
                    modifiers: KeyModifierFlags::new().tap_mut(|x| x.shift()),
                }),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_I.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_SPACE.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_T.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_H.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_E.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_R.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_E.into())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods {
                    key: KEY_1.into(),
                    modifiers: KeyModifierFlags::new().tap_mut(|x| x.shift()),
                }),
            ],
            CustomError { input: "", expected: vec![] }
        )));

        assert_eq!(key_sequence("a{b down}"), nom_ok((
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_A.into())),
                ParsedKeyAction::KeyAction(KeyActionWithMods {
                    key: KEY_B.into(),
                    value: TYPE_DOWN,
                    modifiers: KeyModifierFlags::new(),
                }),
            ],
            CustomError { input: "", expected: vec![] }
        )));

        assert_eq!(key_sequence("a{b down}c"), nom_ok((
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_A.into())),
                ParsedKeyAction::KeyAction(KeyActionWithMods::new(KEY_B.into(), TYPE_DOWN, KeyModifierFlags::new())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_C.into())),
            ],
            CustomError { input: "", expected: vec![] }
        )));

        assert_eq!(key_sequence("{shift down}a{shift up}"), nom_ok((
            vec![
                ParsedKeyAction::KeyAction(KeyActionWithMods::new(KEY_LEFTSHIFT.into(), TYPE_DOWN, KeyModifierFlags::new())),
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new(KEY_A.into())),
                ParsedKeyAction::KeyAction(KeyActionWithMods::new(KEY_LEFTSHIFT.into(), TYPE_UP, KeyModifierFlags::new())),
            ],
            CustomError { input: "", expected: vec![] }
        )));
    }

    #[test]
    fn sequence_special_chars() {
        assert_eq!(key_sequence("{relative X 55}"), nom_ok((
            vec![
                ParsedKeyAction::Action(KeyAction::new(Key { event_code: EventCode::EV_REL(REL_X) }, 55)),
            ],
            CustomError { input: "", expected: vec![] }
        )));
    }


    #[test]
    fn sequence_escaped_special_chars() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();
        let key_sequence = key_sequence_utf(Some(&t));

        assert_eq!(key_sequence("\\{"), nom_ok((
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(
                    KEY_LEFTBRACE.into(), KeyModifierFlags::new().tap_mut(|x| x.shift()),
                )),
            ],
            CustomError { input: "", expected: vec![] },
        )));

        assert_eq!(key_sequence("\\}"), nom_ok((
            vec![
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(
                    KEY_RIGHTBRACE.into(), KeyModifierFlags::new().tap_mut(|x| x.shift()),
                )),
            ],
            CustomError { input: "", expected: vec![] },
        )));
    }

    #[test]
    fn sequence_invalid_multiple_keys_in_special_group() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();
        let key_sequence = key_sequence_utf(Some(&t));

        assert_nom_err(key_sequence("{abc}"), "{abc}");
    }
}