use crate::parsing::action_state::*;
use crate::xkb::XKBTransformer;

use super::*;

#[derive(PartialEq, Debug, Clone)]
pub enum ParsedKeyAction {
    KeyAction(KeyActionWithMods),
    KeyClickAction(KeyClickActionWithMods),
    Action(KeyAction),
}

pub trait ParsedKeyActionVecExt {
    fn to_key_actions(self) -> Vec<KeyAction>;
    fn to_input_ev(self) -> Vec<EvdevInputEvent>;
}

impl ParsedKeyActionVecExt for Vec<ParsedKeyAction> {
    fn to_key_actions(self) -> Vec<KeyAction> {
        // TODO keep track of modifier keys and revert to a sane state after every action
        self.into_iter()
            .fold(vec![], |mut acc, v| match v {
                ParsedKeyAction::KeyAction(action) => {
                    if action.modifiers.ctrl { acc.push(KeyAction::new(KEY_LEFTCTRL.into(), TYPE_DOWN)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(KEY_LEFTSHIFT.into(), TYPE_DOWN)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(KEY_LEFTALT.into(), TYPE_DOWN)); }
                    if action.modifiers.right_alt { acc.push(KeyAction::new(KEY_RIGHTALT.into(), TYPE_DOWN)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(KEY_LEFTMETA.into(), TYPE_DOWN)); }
                    acc.push(KeyAction::new(action.key, action.value));
                    if action.modifiers.ctrl { acc.push(KeyAction::new(KEY_LEFTCTRL.into(), TYPE_UP)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(KEY_LEFTSHIFT.into(), TYPE_UP)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(KEY_LEFTALT.into(), TYPE_UP)); }
                    if action.modifiers.right_alt { acc.push(KeyAction::new(KEY_RIGHTALT.into(), TYPE_UP)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(KEY_LEFTMETA.into(), TYPE_UP)); }
                    acc
                }
                ParsedKeyAction::KeyClickAction(action) => {
                    if action.modifiers.ctrl { acc.push(KeyAction::new(KEY_LEFTCTRL.into(), TYPE_DOWN)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(KEY_LEFTSHIFT.into(), TYPE_DOWN)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(KEY_LEFTALT.into(), TYPE_DOWN)); }
                    if action.modifiers.right_alt { acc.push(KeyAction::new(KEY_RIGHTALT.into(), TYPE_DOWN)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(KEY_LEFTMETA.into(), TYPE_DOWN)); }
                    acc.push(KeyAction::new(action.key, TYPE_DOWN));
                    acc.push(KeyAction::new(action.key, TYPE_UP));
                    if action.modifiers.ctrl { acc.push(KeyAction::new(KEY_LEFTCTRL.into(), TYPE_UP)); }
                    if action.modifiers.shift { acc.push(KeyAction::new(KEY_LEFTSHIFT.into(), TYPE_UP)); }
                    if action.modifiers.alt { acc.push(KeyAction::new(KEY_LEFTALT.into(), TYPE_UP)); }
                    if action.modifiers.right_alt { acc.push(KeyAction::new(KEY_RIGHTALT.into(), TYPE_UP)); }
                    if action.modifiers.meta { acc.push(KeyAction::new(KEY_LEFTMETA.into(), TYPE_UP)); }
                    acc
                }
                ParsedKeyAction::Action(action) => {
                    acc.push(action);
                    acc
                }
            })
    }

    fn to_input_ev(self) -> Vec<EvdevInputEvent> {
        self.to_key_actions()
            .into_iter()
            .map(|x| x.to_input_ev())
            .collect()
    }
}

pub fn complex_key_action_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ParseResult<&str, ((Key, Option<KeyModifierFlags>), Option<i32>)> {
    move |input: &str| {
        alt((
            // key action with state - a down
            map(
                tuple((
                    key_utf(transformer),
                    multispace1,
                    action_state,
                )),
                |((key, mods), _, state)| ((key, Some(mods)), Some(state)),
            ),

            // motion action - relative X 20
            map(
                motion_action,
                |action| ((action.key, None), Some(action.value)),
            ),
        ))(input)
    }
}

pub fn single_key_action_with_flags(input: &str) -> ParseResult<&str, ParsedKeyAction> {
    single_key_action_utf_with_flags_utf(None)(input)
}

pub fn single_key_action_utf_with_flags_utf<'a>(
    transformer: Option<&'a XKBTransformer>,
) -> Box<dyn Fn(&'a str) -> ParseResult<&str, ParsedKeyAction> + 'a> {
    Box::new(move |input: &str| {
        map_res(
            tuple((
                key_flags,
                alt((
                    single_key_action_utf(transformer),
                    surrounded_group("{", "}", single_key_action_utf(transformer)),
                )),
            )),
            |(flags, mut action)| {
                match &mut action {
                    ParsedKeyAction::KeyAction(action) => { action.modifiers.apply_from(&flags) }
                    ParsedKeyAction::KeyClickAction(action) => { action.modifiers.apply_from(&flags) }
                    // TODO figure out how to not accept flags on this
                    ParsedKeyAction::Action(_) => {}
                }

                Ok::<ParsedKeyAction, CustomError<&str>>(action)
            },
        )(input)
    })
}

pub fn single_key_action_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ParseResult<&str, ParsedKeyAction> {
    move |input: &str| {
        map_res(
            alt((
                // any complex action
                complex_key_action_utf(transformer),

                // escaped special char - \\{
                map(tuple((tag_custom("\\"), key_utf(transformer))), |(_, (key, mods))| ((key, Some(mods)), None)),

                // special key
                map_res(
                    recognize(ident),
                    |input| {
                        let (_, (key, mods)) = key_utf(transformer)(input)?;
                        Ok::<_, nom::Err<CustomError<&str>>>(((key, Some(mods)), None))
                    },
                ),

                // any 1 char except special ones
                map_res(
                    recognize(none_of("{}")),
                    |input| {
                        let (_, (key, mods)) = key_utf(transformer)(input)?;
                        Ok::<_, nom::Err<CustomError<&str>>>(((key, Some(mods)), None))
                    },
                ),
            )),
            |((key, mods), value)| {
                let action = match value {
                    Some(value) => {
                        match mods {
                            Some(mods) => ParsedKeyAction::KeyAction(KeyActionWithMods::new(key, value, mods)),
                            None => { ParsedKeyAction::Action(KeyAction::new(key, value)) }
                        }
                    }
                    None => {
                        ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(key, mods.unwrap_or_default()))
                    }
                };

                Ok::<ParsedKeyAction, CustomError<&str>>(action)
            },
        )(input)
    }
}

pub fn key_action(input: &str) -> ParseResult<&str, ParsedKeyAction> {
    key_action_utf(None)(input)
}

pub fn key_action_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ParseResult<&str, ParsedKeyAction> {
    move |input: &str| {
        map_res(
            alt((
                // key action with state {a down}
                surrounded_group("{", "}", complex_key_action_utf(transformer)),

                // key action without state - {a}
                map(
                    surrounded_group("{", "}", key_utf(transformer)),
                    |(key, mods)| ((key, Some(mods)), None),
                ),

                // escaped special char - \\{
                map(tuple((tag_custom("\\"), key_utf(transformer))), |(_, (key, mods))| ((key, Some(mods)), None)),

                // any 1 char except special ones
                map_res(
                    recognize(none_of("{}")),
                    |input| {
                        let (_, (key, mods)) = key_utf(transformer)(input)?;
                        Ok::<_, nom::Err<CustomError<&str>>>(((key, Some(mods)), None))
                    },
                ),
            )),
            |((key, mods), value)| {
                let action = match value {
                    Some(value) => {
                        match mods {
                            Some(mods) => ParsedKeyAction::KeyAction(KeyActionWithMods::new(key, value, mods)),
                            None => { ParsedKeyAction::Action(KeyAction::new(key, value)) }
                        }
                    }
                    None => {
                        ParsedKeyAction::KeyClickAction(KeyClickActionWithMods::new_with_mods(key, mods.unwrap_or_default()))
                    }
                };

                Ok::<ParsedKeyAction, CustomError<&str>>(action)
            },
        )(input)
    }
}


#[cfg(test)]
mod tests {
    use evdev_rs::enums::EV_REL;
    use super::*;

    #[test]
    fn action_with_state() {
        assert_eq!(key_action("{a down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("KEY_A").unwrap(), 1, KeyModifierFlags::new())
        )));

        assert_eq!(key_action("{btn_forward down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("BTN_FORWARD").unwrap(), 1, KeyModifierFlags::new())
        )));

        assert_eq!(key_action("{shift down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("KEY_LEFTSHIFT").unwrap(), 1, KeyModifierFlags::new())
        )));

        assert_eq!(key_action("{relative X 99}"), nom_ok(ParsedKeyAction::Action(
            KeyAction { key: Key { event_code: EventCode::EV_REL(REL_X) }, value: 99 }
        )));

        assert_eq!(key_action("{absolute X 99}"), nom_ok(ParsedKeyAction::Action(
            KeyAction { key: Key { event_code: EventCode::EV_ABS(ABS_X) }, value: 99 }
        )));
    }

    #[test]
    fn action_with_mods() {
        assert_eq!(single_key_action_with_flags("+a down"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("KEY_A").unwrap(), 1, KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));

        assert_eq!(single_key_action_with_flags("!j down"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("KEY_J").unwrap(), 1, KeyModifierFlags::new().tap_mut(|v| v.alt()))
        )));
    }

    #[test]
    fn action_utf() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_eq!(key_action_utf(Some(&t))("{: down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("semicolon").unwrap(), 1,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));

        assert_eq!(key_action_utf(Some(&t))("{^ down}"), nom_ok(ParsedKeyAction::KeyAction(
            KeyActionWithMods::new(Key::from_str("6").unwrap(), 1,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));

        assert_eq!(key_action_utf(Some(&t))("^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(Key::from_str("6").unwrap(),
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )
        )));
    }

    #[test]
    fn single_key_action_input() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_nom_err(key_action_utf(Some(&t))("{"), "{");

        assert_eq!(single_key_action_utf_with_flags_utf(Some(&t))("\\^"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(Key::from_str("6").unwrap(),
                KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));

        assert_eq!(single_key_action_utf_with_flags_utf(Some(&t))("\\{"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(Key::from_str("leftbrace").unwrap(),
                KeyModifierFlags::new().tap_mut(|v| v.shift()))
        )));

        assert_eq!(single_key_action_utf_with_flags_utf(Some(&t))("page_down"), nom_ok(ParsedKeyAction::KeyClickAction(
            KeyClickActionWithMods::new_with_mods(KEY_PAGEDOWN.into(), KeyModifierFlags::new())
        )));
    }

    #[test]
    fn invalid_action_multiple_keys_in_special_group() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_nom_err(key_action_utf(Some(&t))("{abc}"), "{abc}");
    }
}