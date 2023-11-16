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
        map_res(
            many1(
                // alt((
                //     // map_res(
                //     //     recognize(tuple((
                //     //         key_flags,
                //     //         tag_custom("{"),
                //     //         terminated(take_until("}"), tag_custom("}"))),
                //     //     )),
                //     //     |input| {
                //     //         let (input, action) = alt((
                //     //             key_action_with_flags_utf(transformer),
                //     //             key_action_utf(transformer),
                //     //         ))(input)?;
                //     //         // TODO properly propagate child error
                //     //         if !input.is_empty() {
                //     //             return Err(make_generic_nom_err_new(input));
                //     //         }
                //     //
                //     //         Ok((input, action))
                //     //     },
                //     // ),
                //     // map(surrounded_group("{", "}", key_action_utf(transformer)), |(_, (_,action))| action),
                //     surrounded_group("{", "}", key_action_utf(transformer)),
                //     // map_res(take(1usize), key_action_utf(transformer)),
                //
                //
                //
                // )),
                key_action_utf(transformer)
            ),
            |seq| {
                // let seq = seq.into_iter()
                //     .map(|(rest, res)| {
                //         if !rest.is_empty() { return Err(make_generic_nom_err_new(input)); }
                //         Ok(res)
                //     })
                //     .fold_ok(vec![], |mut acc, v| {
                //         acc.push(v);
                //         acc
                //     })?;
                Ok::<Vec<ParsedKeyAction>, nom::Err<CustomError<&str>>>(seq)
            },
        )(input)
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
                *KEY_LEFTBRACE,
                KeyModifierFlags::new().tap_mut(|x| x.shift()),
            )),
        ]));
    }

    #[test]
    fn sequence_invalid_multiple_keys_in_special_group() {
        let t = UTFToRawInputTransformer::new(None, Some("us"), None, None);

        assert_nom_err(key_sequence_utf(Some(&t))("{abc}"), "{abc}");
    }
}