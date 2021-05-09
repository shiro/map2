use super::*;
use nom::combinator::map_res;

#[derive(PartialEq, Debug, Clone)]
pub(super) enum ParsedKeyAction {
    KeyAction(KeyActionWithMods),
    KeyClickAction(KeyClickActionWithMods),
}

pub(super) trait ParsedKeyActionVecExt {
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

pub(super) fn key_action(input: &str) -> ResNew<&str, ParsedKeyAction> {
    alt((
        map(tuple((tag_custom("{"), key_with_state, tag_custom("}"))), |(_, (v, _), _)| (v.0, Some(v.1))),
        map(
            alt((
                map(tuple((tag_custom("{"), key, tag_custom("}"))), |v| v.1.0),
                map(key, |v| v.0),
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

pub(super) fn key_action_with_flags(input: &str) -> ResNew<&str, ParsedKeyAction> {
    tuple((
        key_flags,
        key_action,
    ))(input).and_then(|(next, parts)| {
        let flags = parts.0;
        let mut action = parts.1;

        match &mut action.0 {
            ParsedKeyAction::KeyAction(action) => { action.modifiers.apply_from(&flags.0) }
            ParsedKeyAction::KeyClickAction(action) => { action.modifiers.apply_from(&flags.0) }
        }

        Ok((next, (action.0, None)))
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
}