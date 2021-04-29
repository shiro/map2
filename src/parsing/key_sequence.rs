use super::*;
use nom::combinator::{peek, recognize, map_res};
use nom::multi::many1;
use itertools::Itertools;

pub(super) fn key_sequence(mut input: &str) -> Res<&str, Vec<ParsedKeyAction>> {
    context(
        "key_sequence",
        tuple((
            tag("\""),
            many1(
                alt((
                    map_res(take(1usize), key_action),
                    map_res(
                        recognize(terminated(take_until("}"), tag("}"))),
                        key_action,
                    ),
                )),
            ),
            tag("\""),
        )),
    )(input).and_then(|(next, val)| Ok((next, val.1.into_iter()
        .map(|v| {
            if !v.0.is_empty() { return Err(make_generic_nom_err()); }
            Ok(v.1)
            // Ok(33)
            // v.1
        })
        .fold_ok(vec![], |mut acc, v| {
            acc.push(v);
            acc
        })?

                                        //.collect()
    )))
    //
    //
    // let mut actions = vec![];
    // (input, _) = tag("\"")(input)?;
    //
    // loop {
    //     let mut ch = "";
    //     (input, ch) = take(1usize)(input)?;
    //
    //     match peek(take(1usize))(input) {
    //         Ok((_, '{')) => {
    //             let (rest, action) = key_action(key_name)?;
    //             if !rest.is_empty() { return Err(make_generic_nom_err()); }
    //         }
    //         Ok(_) => {}
    //         _ => {}
    //     }
    //
    //     if ch == "\"" { break; }
    //
    //     let mut key_name = ch;
    //
    //     if ch == "{" {
    //         (input, key_name) = take_until("}")(input)?;
    //         (input, _) = take(1usize)(input)?;
    //     }
    //
    //     let (rest, action) = key_action(key_name)?;
    //     if !rest.is_empty() { return Err(make_generic_nom_err()); }
    //
    //     actions.push(action);
    // }
    // Ok((input, actions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_sequence() {
        assert_eq!(key_sequence("\"abc\""), Ok(("", vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_B, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_C, modifiers: KeyModifierFlags::new() }),
        ])));

        assert_eq!(key_sequence("\"a{b down}\""), Ok(("", vec![
            ParsedKeyAction::KeyClickAction(KeyClickActionWithMods { key: *KEY_A, modifiers: KeyModifierFlags::new() }),
            ParsedKeyAction::KeyAction(KeyActionWithMods { key: *KEY_B, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }),
        ])));
    }
}
