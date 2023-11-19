use evdev_rs::enums::EV_KEY;

use crate::xkb::XKBTransformer;

use super::*;

pub fn key_flags(input: &str) -> ResNew2<&str, KeyModifierFlags> {
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
        Ok((next, flags))
    })
}

pub fn key(input: &str) -> ResNew2<&str, (Key, KeyModifierFlags)> {
    key_utf(None)(input)
}

pub fn key_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ResNew2<&'a str, (Key, KeyModifierFlags)> + 'a {
    move |input: &str| {
        alt((
            // multiple ASCII chars
            ident,

            // one arbitrary char
            map(take(1usize), |v: &str| v.to_string())
        ))(input)
            .and_then(|(next, key_name)| {
                let (key, mut flags) = match KEY_ALIAS_TABLE.get(&*key_name.to_uppercase()) {
                    Some(v) => *v,
                    None => {
                        if let Some(transformer) = transformer {
                            let mut seq = transformer.utf_to_raw(key_name.to_string())
                                .map_err(|_| make_generic_nom_err_new(input))?;

                            let key = seq.remove(seq.len() - 1);

                            let mut flags = KeyModifierFlags::new();

                            for ev in seq.iter() {
                                match ev {
                                    Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_LEFTALT) } => { flags.alt(); }
                                    Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_RIGHTALT) } => { flags.right_alt(); }
                                    Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_LEFTSHIFT) } => { flags.shift(); }
                                    Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_RIGHTSHIFT) } => { flags.shift(); }
                                    _ => { unreachable!("unhandled modifier") }
                                }
                            }

                            (key, flags)
                        } else {
                            let mut key_name = key_name.to_uppercase();
                            if !key_name.starts_with("KEY_") && !key_name.starts_with("BTN_") {
                                key_name = "KEY_".to_string()
                                    .tap_mut(|s| s.push_str(&key_name));
                            }

                            let key = Key::from_str(&EventType::EV_KEY, key_name.as_str())
                                .map_err(|_| make_generic_nom_err_new(input))?;

                            (key, KeyModifierFlags::new())
                        }
                    }
                };

                if transformer.is_none() {
                    // only 1 char and it's uppercase
                    let mut it = key_name.chars();
                    if it.next().unwrap().is_uppercase() && it.next().is_none() {
                        flags.shift();
                    }
                }

                Ok((next, (key, flags)))
            })
    }
}

fn key_state(input: &str) -> ResNew2<&str, i32> {
    alt((
        tag("down"), tag("up"), tag("repeat"),
    ))(input).map(|(next, v)| (next, match v.to_uppercase().as_str() {
        "UP" => 0,
        "DOWN" => 1,
        "REPEAT" => 2,
        _ => unreachable!()
    }))
}

pub fn key_with_state(input: &str) -> ResNew2<&str, ((Key, KeyModifierFlags), i32)> {
    key_with_state_utf(None)(input)
}

pub fn key_with_state_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ResNew2<&'a str, ((Key, KeyModifierFlags), i32)> + 'a {
    move |input: &str| {
        map(
            tuple((key_utf(transformer), multispace1, key_state)),
            |(key, _, state)| (key, state),
        )(input)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_key() {
        assert_eq!(key_with_state("a down"), nom_ok((
            (*KEY_A, KeyModifierFlags::new()), 1
        )));
    }

    #[test]
    fn test_key() {
        assert_eq!(key("d"), nom_ok((
            Key::from_str(&EventType::EV_KEY, "KEY_D").unwrap(),
            KeyModifierFlags::new()
        )));

        assert_eq!(key("btn_forward"), nom_ok((
            Key::from_str(&EventType::EV_KEY, "BTN_FORWARD").unwrap(),
            KeyModifierFlags::new()))
        );
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

    #[test]
    fn test_utf_key() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_eq!(key_utf(Some(&t))(":"), nom_ok((
            *KEY_SEMICOLON,
            KeyModifierFlags::new().tap_mut(|x| x.shift())
        )));

        assert_eq!(key_utf(Some(&t))("^"), nom_ok((
            *KEY_6,
            KeyModifierFlags::new().tap_mut(|x| x.shift())
        )));
    }

    #[test]
    fn key_special() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        assert_eq!(key_utf(Some(&t))("SHIFT"), nom_ok((*KEY_LEFTSHIFT, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("BACKSPACE"), nom_ok((*KEY_BACKSPACE, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("LEFT"), nom_ok((*KEY_LEFT, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("RIGHT"), nom_ok((*KEY_RIGHT, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("UP"), nom_ok((*KEY_UP, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("DOWN"), nom_ok((*KEY_DOWN, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("PAGE_DOWN"), nom_ok((*KEY_PAGEDOWN, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("PAGE_UP"), nom_ok((*KEY_PAGEUP, KeyModifierFlags::new())));

        assert_eq!(key_utf(Some(&t))("BTN_LEFT"), nom_ok((*BTN_LEFT, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("BTN_RIGHT"), nom_ok((*BTN_RIGHT, KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("BTN_MIDDLE"), nom_ok((*BTN_MIDDLE, KeyModifierFlags::new())));
    }

    #[test]
    fn invalid_key_multiple_chars() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        // assert_eq!(key_utf(Some(&t))("ab"), nom_err("ab"));
        assert_nom_err(key_utf(Some(&t))("ab"), "ab");
    }
}