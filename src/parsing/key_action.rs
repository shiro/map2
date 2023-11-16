use crate::xkb::UTFToRawInputTransformer;

use super::*;

#[derive(PartialEq, Debug, Clone)]
pub enum ParsedKeyAction {
    KeyAction(KeyActionWithMods),
    KeyClickAction(KeyClickActionWithMods),
}

pub trait ParsedKeyActionVecExt {
    fn to_key_actions(self) -> Vec<KeyAction>;
}

impl ParsedKeyActionVecExt for Vec<ParsedKeyAction> {
    fn to_key_actions(self) -> Vec<KeyAction> {
        // TODO keep track of modifier keys and revert to a sane state after every action
        self.into_iter()
            .fold(vec![], |mut acc, v| match v {
                ParsedKeyAction::KeyAction(action) => {
                    if action.modifiers.ctrl { acc.push(KeyAction::new(*KEY_LEFT_CTRL, TYPE_DOWN)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_DOWN)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(*KEY_LEFT_ALT, TYPE_DOWN)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(*KEY_LEFT_META, TYPE_DOWN)); }
                    acc.push(KeyAction::new(action.key, action.value));
                    if action.modifiers.ctrl { acc.push(KeyAction::new(*KEY_LEFT_CTRL, TYPE_UP)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_UP)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(*KEY_LEFT_ALT, TYPE_UP)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(*KEY_LEFT_META, TYPE_UP)); }
                    acc
                }
                ParsedKeyAction::KeyClickAction(action) => {
                    if action.modifiers.ctrl { acc.push(KeyAction::new(*KEY_LEFT_CTRL, TYPE_DOWN)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_DOWN)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(*KEY_LEFT_ALT, TYPE_DOWN)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(*KEY_LEFT_META, TYPE_DOWN)); }
                    acc.push(KeyAction::new(action.key, TYPE_DOWN));
                    acc.push(KeyAction::new(action.key, TYPE_UP));
                    if action.modifiers.ctrl { acc.push(KeyAction::new(*KEY_LEFT_CTRL, TYPE_UP)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_UP)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(*KEY_LEFT_ALT, TYPE_UP)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(*KEY_LEFT_META, TYPE_UP)); }
                    acc
                }
            })
    }
}


pub fn key_action(input: &str) -> ResNew2<&str, ParsedKeyAction> {
    key_action_utf(None)(input)
}

pub fn key_action_utf<'a>(
    transformer: Option<&'a UTFToRawInputTransformer>
) -> impl Fn(&'a str) -> ResNew2<&'a str, ParsedKeyAction> {
    move |input: &str| {
        map_res(
            alt((
                // with state
                map(surrounded_group("{", "}", key_with_state_utf(transformer)),
                    |(key, state)| (key, Some(state)),
                ),

                // no state - {KEY}
                map(surrounded_group("{", "}", key_utf(transformer)),
                    |key| (key, None),
                ),

                // escaped special char
                map(tuple((tag_custom("\\"), key_utf(transformer))), |(_, key)| (key, None)),

                // any 1 char except special ones
                map_res(
                    recognize(none_of("{}")),
                    |input| {
                        let (_, key) = key_utf(transformer)(input)?;
                        Ok::<_, nom::Err<CustomError<&str>>>((key, None))
                    },
                ),
            )), |((key, mods), state)| {
                let action = match state {
                    Some(state) => {
                        ParsedKeyAction::KeyAction(KeyActionWithMods::new(key, state, mods))
                    }
                    None => {
                        ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(key, mods))
                    }
                };

                Ok::<ParsedKeyAction, CustomError<&str>>(action)
            })(input)
    }
}

pub fn key_action_with_flags(input: &str) -> ResNew2<&str, ParsedKeyAction> {
    key_action_with_flags_utf(None)(input)
}

pub fn key_action_with_flags_utf<'a>(
    transformer: Option<&'a UTFToRawInputTransformer>,
) -> Box<dyn Fn(&'a str) -> ResNew2<&'a str, ParsedKeyAction> + 'a> {
    Box::new(move |input: &str| {
        map_res(
            tuple((
                key_flags,
                key_action_utf(transformer),
            )),
            |parts| {
                let flags = parts.0;
                let mut action = parts.1;

                match &mut action {
                    ParsedKeyAction::KeyAction(action) => { action.modifiers.apply_from(&flags) }
                    ParsedKeyAction::KeyClickAction(action) => { action.modifiers.apply_from(&flags) }
                }

                Ok::<ParsedKeyAction, CustomError<&str>>(action)
            },
        )(input)
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_with_state() {
        assert_eq!(key_action("{a down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap(), 1, KeyModifierFlags::new())
        )));

        assert_eq!(key_action("{btn_forward down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "BTN_FORWARD").unwrap(), 1, KeyModifierFlags::new())
        )));

        assert_eq!(key_action("{shift down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_LEFTSHIFT").unwrap(), 1, KeyModifierFlags::new())
        )));
    }

    #[test]
    fn action_with_mods() {
        assert_eq!(key_action_with_flags("+{a down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap(), 1, KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));

        assert_eq!(key_action_with_flags("!{j down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_J").unwrap(), 1, KeyModifierFlags::new().tap_mut(|v| v.alt()))
        )));
    }

    #[test]
    fn action_utf() {
        let t = UTFToRawInputTransformer::new(None, Some("us"), None, None);

        assert_eq!(key_action_utf(Some(&t))("{: down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(*KEY_SEMICOLON, 1,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));

        assert_eq!(key_action_utf(Some(&t))("{^ down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(*KEY_6, 1,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));

        assert_eq!(key_action_utf(Some(&t))("^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(*KEY_6,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));
    }

    #[test]
    fn action_handle_special_chars() {
        let t = UTFToRawInputTransformer::new(None, Some("us"), None, None);

        assert_nom_err(key_action_utf(Some(&t))("{"), "{");

        assert_eq!(key_action_with_flags_utf(Some(&t))("\\^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(*KEY_6,
                KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));

        assert_eq!(key_action_with_flags_utf(Some(&t))("\\{"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(*KEY_LEFTBRACE,
                KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));
    }

    #[test]
    fn invalid_action_multiple_keys_in_special_group() {
        let t = UTFToRawInputTransformer::new(None, Some("us"), None, None);

        assert_nom_err(key_action_utf(Some(&t))("{abc}"), "{abc}");
    }
}