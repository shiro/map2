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


pub fn key_action(input: &str) -> ResNew<&str, ParsedKeyAction> {
    key_action_utf(None)(input)
}

pub fn key_action_utf<'a>(
    transformer: Option<&'a UTFToRawInputTransformer>
) -> impl Fn(&'a str) -> ResNew<&'a str, ParsedKeyAction> {
    move |input: &str| {
        alt((
            map(tuple((tag_custom("{"), key_with_state_utf(transformer), tag_custom("}"))), |(_, (v, _), _)| (v.0, Some(v.1))),
            map(
                alt((
                    map(tuple((tag_custom("{"), key_utf(transformer), tag_custom("}"))), |v| v.1.0),
                    map(key_utf(transformer), |v| v.0),
                )),
                |v| (v, None),
            ),
        ))(input).and_then(|(next, ((key, mods), state))| {
            let action = match state {
                Some(state) => {
                    ParsedKeyAction::KeyAction(KeyActionWithMods::new(key, state, mods))
                }
                None => {
                    ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(key, mods))
                }
            };

            Ok((next, (action, None)))
        })
    }
}

pub fn key_action_with_flags(input: &str) -> ResNew<&str, ParsedKeyAction> {
    key_action_with_flags_utf(None)(input)
}

pub fn key_action_with_flags_utf<'a>(
    transformer: Option<&'a UTFToRawInputTransformer>,
) -> Box<dyn Fn(&'a str) -> ResNew<&'a str, ParsedKeyAction> + 'a> {
    Box::new(move |input: &str| {
        tuple((
            key_flags,
            key_action_utf(transformer),
        ))(input).and_then(|(next, parts)| {
            let flags = parts.0;
            let mut action = parts.1;

            match &mut action.0 {
                ParsedKeyAction::KeyAction(action) => { action.modifiers.apply_from(&flags.0) }
                ParsedKeyAction::KeyClickAction(action) => { action.modifiers.apply_from(&flags.0) }
            }

            Ok((next, (action.0, None)))
        })
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_action() {
        assert_eq!(key_action("{a down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap(), 1, KeyModifierFlags::new())
        )));

        assert_eq!(key_action("{btn_forward down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "BTN_FORWARD").unwrap(), 1, KeyModifierFlags::new())
        )));
    }

    #[test]
    fn test_flags() {
        assert_eq!(key_action_with_flags("+{a down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap(), 1, KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));

        assert_eq!(key_action_with_flags("!{j down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_J").unwrap(), 1, KeyModifierFlags::new().tap_mut(|v| v.alt()))
        )));
    }

    #[test]
    fn parse_key_action_utf() {
        let t = UTFToRawInputTransformer::new(None, Some("rabbit"), None, None);
        //
        // assert_eq!(key_action_utf(Some(&t))("{š down}"), nom_ok(ParsedKeyAction::KeyAction(
        //     KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_S").unwrap(), 1,
        //         KeyModifierFlags::new().tap_mut(|x| x.right_alt()),
        //     )
        // )));
        //
        // assert_eq!(key_action_utf(Some(&t))("{^ down}"), nom_ok(ParsedKeyAction::KeyAction(
        //     KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap(), 1,
        //         KeyModifierFlags::new().tap_mut(|x| x.shift()),
        //     )
        // )));
        //
        // assert_eq!(key_action_utf(Some(&t))("^"), nom_ok(ParsedKeyAction::KeyClickAction(
        //     KeyClickActionWithMods::new_with_mods(Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap(),
        //         KeyModifierFlags::new().tap_mut(|x| x.shift()),
        //     )
        // )));
        //
        assert_eq!(key_action_with_flags_utf(Some(&t))("\\^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap(),
                KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));
    }
}