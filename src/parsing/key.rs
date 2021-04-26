use super::*;


pub(super) fn key_flags(input: &str) -> Res<&str, KeyModifierFlags> {
    context("key_flags", many0(one_of("^!+#")))(input).and_then(|(next, val)| {
        let mut flags = KeyModifierFlags::new();
        for v in val {
            match v {
                '!' => { if !flags.alt { flags.alt(); } else { return Err(make_generic_nom_err()); } }
                '^' => { if !flags.ctrl { flags.ctrl(); } else { return Err(make_generic_nom_err()); } }
                '+' => { if !flags.shift { flags.shift(); } else { return Err(make_generic_nom_err()); } }
                '#' => { if !flags.meta { flags.meta(); } else { return Err(make_generic_nom_err()); } }
                _ => unreachable!()
            }
        };
        Ok((next, flags))
    })
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(super) enum ParsedSingleKey {
    Key(Key),
    CapitalKey(Key),
}

pub(super) fn key(input: &str) -> Res<&str, ParsedSingleKey> {
    context("key", ident)(input)
        .and_then(|(next, val)| {
            let mut key_name = val.to_uppercase();

            if !key_name.starts_with("KEY_") && !key_name.starts_with("BTN_") {
                key_name = "KEY_".to_string()
                    .tap_mut(|s| s.push_str(&key_name));
            }

            let key = Key::from_str(&EventType::EV_KEY, key_name.as_str())
                .map_err(|_| make_generic_nom_err())?;

            // only 1 char and it's uppercase
            let mut it = val.chars();
            if it.next().unwrap().is_uppercase() && it.next().is_none() {
                return Ok((next, ParsedSingleKey::CapitalKey(key.clone())));
            }

            Ok((next, ParsedSingleKey::Key(key.clone())))
        })
}

fn key_state(input: &str) -> Res<&str, i32> {
    context("key_state", alt((
        tag("down"), tag("up"),
    )))(input).map(|(next, v)| (next, match v.to_uppercase().as_str() {
        "UP" => 0,
        "DOWN" => 1,
        _ => unreachable!()
    }))
}

pub(super) fn key_with_state(input: &str) -> Res<&str, (ParsedSingleKey, i32)> {
    context(
        "special_key", tuple((
            tag("{"),
            multispace0,
            key,
            multispace0,
            key_state,
            multispace0,
            tag("}"),
        )))(input)
        .map(|(next, val)| (next, (val.2, val.4)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_key() {
        assert_eq!(key_with_state("{a down}"), Ok(("", (
            ParsedSingleKey::Key(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap()),
            1,
        ))));
    }

    #[test]
    fn test_key() {
        assert_eq!(key("d"), Ok(("", ParsedSingleKey::Key(
            Key::from_str(&EventType::EV_KEY, "KEY_D").unwrap())
        )));

        assert_eq!(key("btn_forward"), Ok(("", ParsedSingleKey::Key(
            Key::from_str(&EventType::EV_KEY, "BTN_FORWARD").unwrap())
        )));
    }

    #[test]
    fn test_key_flags() {
        assert_eq!(key_flags("!"), Ok(("", *KeyModifierFlags::new().alt())));
        assert_eq!(key_flags("^"), Ok(("", *KeyModifierFlags::new().ctrl())));
        assert_eq!(key_flags("+"), Ok(("", *KeyModifierFlags::new().shift())));
        assert_eq!(key_flags("#"), Ok(("", *KeyModifierFlags::new().meta())));

        assert_eq!(key_flags("!#"), Ok(("", *KeyModifierFlags::new().alt().meta())));
        assert_eq!(key_flags("#!"), Ok(("", *KeyModifierFlags::new().alt().meta())));
        assert_eq!(key_flags("#a!"), Ok(("a!", *KeyModifierFlags::new().meta())));
    }
}
