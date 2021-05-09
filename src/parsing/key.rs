use super::*;

pub(super) fn key_flags(input: &str) -> ResNew<&str, KeyModifierFlags> {
    many0(one_of("^!+#"))(input).and_then(|(next, val)| {
        let mut flags = KeyModifierFlags::new();
        for v in val {
            match v {
                '!' => { if !flags.alt { flags.alt(); } else { return Err(make_generic_nom_err_new(input)); } }
                '^' => { if !flags.ctrl { flags.ctrl(); } else { return Err(make_generic_nom_err_new(input)); } }
                '+' => { if !flags.shift { flags.shift(); } else { return Err(make_generic_nom_err_new(input)); } }
                '#' => { if !flags.meta { flags.meta(); } else { return Err(make_generic_nom_err_new(input)); } }
                _ => unreachable!()
            }
        };
        Ok((next, (flags, None)))
    })
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(super) enum ParsedSingleKey {
    Key(Key),
    CapitalKey(Key),
}

pub(super) fn key(input: &str) -> ResNew<&str, ParsedSingleKey> {
    alt(( // multiple asci chars or 1 arbitrary char
          map(ident, |v| v.0),
          map(take(1usize), |v: &str| v.to_string())
    ))(input)
        .and_then(|(next, val)| {
            let mut key_name = val.to_uppercase();

            let key = match KEY_ALIAS_TABLE.get(&*key_name) {
                Some(key) => *key,
                None => {
                    if !key_name.starts_with("KEY_") && !key_name.starts_with("BTN_") {
                        key_name = "KEY_".to_string()
                            .tap_mut(|s| s.push_str(&key_name));
                    }

                    Key::from_str(&EventType::EV_KEY, key_name.as_str())
                        .map_err(|_| make_generic_nom_err_new(input))?
                }
            };

            // only 1 char and it's uppercase
            let mut it = val.chars();
            if it.next().unwrap().is_uppercase() && it.next().is_none() {
                return Ok((next, (ParsedSingleKey::CapitalKey(key.clone()), None)));
            }

            Ok((next, (ParsedSingleKey::Key(key.clone()), None)))
        })
}

fn key_state(input: &str) -> ResNew<&str, i32> {
    alt((
        tag("down"), tag("up"),
    ))(input).map(|(next, v)| (next, match v.to_uppercase().as_str() {
        "UP" => (0, None),
        "DOWN" => (1, None),
        _ => unreachable!()
    }))
}

pub(super) fn key_with_state(input: &str) -> ResNew<&str, (ParsedSingleKey, i32)> {
    tuple((
        key,
        ws0,
        key_state,
    ))(input)
        .map(|(next, val)| (next, ((val.0.0, val.2.0), None)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_key() {
        assert_eq!(key_with_state("a down"), nom_ok((
            ParsedSingleKey::Key(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap()),
            1,
        )));
    }

    #[test]
    fn test_key() {
        assert_eq!(key("d"), nom_ok(ParsedSingleKey::Key(
            Key::from_str(&EventType::EV_KEY, "KEY_D").unwrap())
        ));

        assert_eq!(key("btn_forward"), nom_ok(ParsedSingleKey::Key(
            Key::from_str(&EventType::EV_KEY, "BTN_FORWARD").unwrap())
        ));
    }

    #[test]
    fn test_key_flags() {
        assert_eq!(key_flags("!"), nom_ok(KeyModifierFlags::new().tap_mut(|v| v.alt())));
        assert_eq!(key_flags("^"), nom_ok(KeyModifierFlags::new().tap_mut(|v| v.ctrl())));
        assert_eq!(key_flags("+"), nom_ok(KeyModifierFlags::new().tap_mut(|v| v.shift())));
        assert_eq!(key_flags("#"), nom_ok(KeyModifierFlags::new().tap_mut(|v| v.meta())));

        assert_eq!(key_flags("!#"), nom_ok(KeyModifierFlags::new().tap_mut(|v| {
            v.alt();
            v.meta()
        })));
        assert_eq!(key_flags("#!"), nom_ok(KeyModifierFlags::new()
            .tap_mut(|v| {
                v.alt();
                v.meta();
            })));
        assert_eq!(key_flags("#a!"), nom_ok_rest("a!", KeyModifierFlags::new().tap_mut(|v| v.meta())));
    }
}
