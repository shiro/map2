use nom::combinator::{map_res, recognize};
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

fn foo<'a, Output>(
    from_token: &'a str,
    to_token: &'a str,
    parser: impl Fn(&'a str) -> ResNew2<&'a str, Output> + 'a,
) -> Box<dyn Fn(&'a str) -> ResNew<&'a str, Output> + 'a> {
    Box::new(move |input| {
        // map(
            map_res(
                tuple((
                    tag_custom(from_token),
                    terminated(take_until(to_token), tag_custom(to_token))
                )),
                |(_, input)| {
                    let (input, res) = parser(input)?;
                    if !input.is_empty() { return Err(make_generic_nom_err_new(input)); }
                    Ok((res, None))
                },
            // ),
            // |(v, _)| v,
        )(input)
    })
}

pub fn key_action(input: &str) -> ResNew<&str, ParsedKeyAction> {
    key_action_utf(None)(input)
}

pub fn key_action_utf<'a>(
    transformer: Option<&'a UTFToRawInputTransformer>
) -> impl Fn(&'a str) -> ResNew<&'a str, ParsedKeyAction> {
    move |input: &str| {
        alt((
            // with state
            // map_res(
            //     tuple((
            //         tag_custom("{"),
            //         terminated(take_until("}"), tag_custom("}"))),
            //     ),
            //     |(_, input)| {
            //         let (input, (action, _)) = key_with_state_utf(transformer)(input)?;
            //         if !input.is_empty() { return Err(make_generic_nom_err_new(input)); }
            //         Ok((action.0, Some(action.1)))
            //     },
            // ),
            // map(foo("{", "}",
            //     key_with_state_utf(transformer)),
            //     |(key, state)| (key, Some(state)),
            // ),
            // foo("{", "}",
            //     key_with_state_utf(transformer))

            map_res(
                recognize(tuple((
                    tag_custom("{"),
                    terminated(take_until("}"), tag_custom("}"))),
                )),
                |input| {
                    let (input, (action, _)) = key_with_state_utf(transformer)(input)?;
                    if !input.is_empty() { return Err(make_generic_nom_err_new(input)); }
                    Ok((action.0, Some(action.1)))
                },
            ),

            // no state
            map(
                alt((
                    // {KEY}
                    map_res(
                        recognize(tuple((
                            tag_custom("{"),
                            terminated(take_until("}"), tag_custom("}"))),
                        )),
                        |input| {
                            let (input, (action, err)) = key_utf(transformer)(input)?;
                            if !input.is_empty() { return Err(make_generic_nom_err_new(input)); }
                            Ok(action)
                        },
                    ),
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

        assert_eq!(key_action("{shift down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_LEFTSHIFT").unwrap(), 1, KeyModifierFlags::new())
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

        assert_eq!(key_action_utf(Some(&t))("{š down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_S").unwrap(), 1,
                KeyModifierFlags::new().tap_mut(|x| x.right_alt()),
            )
        )));

        assert_eq!(key_action_utf(Some(&t))("{^ down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap(), 1,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));

        assert_eq!(key_action_utf(Some(&t))("^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap(),
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));

        assert_eq!(key_action_with_flags_utf(Some(&t))("\\^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(Key::from_str(&EventType::EV_KEY, "KEY_6").unwrap(),
                KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));
    }

    #[test]
    fn invalid_action_multiple_keys_in_special_group() {
        let t = UTFToRawInputTransformer::new(None, Some("rabbit"), None, None);

        assert_nom_err(key_action_utf(Some(&t))("{abc}"), "{abc}");
    }
}