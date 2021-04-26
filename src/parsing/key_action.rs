use super::*;

#[derive(PartialEq, Debug, Clone)]
pub(super) enum ParsedKeyAction {
    KeyAction(KeyActionWithMods),
    KeyClickAction(KeyClickActionWithMods),
    KeySequence(Vec<Expr>),
}

pub(super) fn key_action(input: &str) -> Res<&str, ParsedKeyAction> {
    context(
        "key_action",
        tuple((
            key_flags,
            alt((
                map(key, |v| (v, None)),
                map(key_with_state, |v| (v.0, Some(v.1)))
            )),
        )),
    )(input).and_then(|(next, parts)| {
        let mut mods = parts.0;
        let key;
        let state = parts.1.1;

        match parts.1.0 {
            ParsedSingleKey::Key(k) => { key = k; }
            ParsedSingleKey::CapitalKey(k) => {
                mods.shift();
                key = k;
            }
        }

        let action = match state {
            Some(state) => {
                ParsedKeyAction::KeyAction(KeyActionWithMods::new(key, state, mods))
            }
            None => {
                ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(key, mods))
            }
        };

        Ok((next, action))
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_action() {
        assert_eq!(key_action("{a down}"), Ok(("", ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap(), 1, KeyModifierFlags::new())
        ))));

        assert_eq!(key_action("{btn_forward down}"), Ok(("", ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "BTN_FORWARD").unwrap(), 1, KeyModifierFlags::new())
        ))));
    }
}