use evdev_rs::enums::EV_KEY;

use crate::xkb::XKBTransformer;

use super::*;

pub fn key_flags(input: &str) -> ParseResult<&str, KeyModifierFlags> {
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

pub fn key(input: &str) -> ParseResult<&str, (Key, KeyModifierFlags)> {
    key_utf(None)(input)
}

pub fn key_utf<'a>(
    transformer: Option<&'a XKBTransformer>
) -> impl Fn(&'a str) -> ParseResult<&str, (Key, KeyModifierFlags)> {
    move |input: &str| {
        map_res(
            alt((
                // multiple ASCII chars
                ident,

                // one arbitrary char
                map(take(1usize), |v: &str| v.to_string())
            )),
            |key_name| {
                let (key, mut flags) = match KEY_ALIAS_TABLE.get(&*key_name.to_uppercase()) {
                    Some(v) => *v,
                    None => {
                        // try XKB conversion
                        if let Some(transformer) = transformer {
                            if let Ok(res) = resolve_key_utf8(&key_name, transformer) {
                                return Ok::<_, nom::Err<CustomError<&str>>>(res);
                            }
                        }

                        // fall back to libevdev resolution
                        let key = Key::from_str(&key_name)
                            .map_err(|_| make_generic_nom_err_new(input))?;

                        return Ok((key, KeyModifierFlags::new()))
                    }
                };

                Ok((key, flags))
            },
        )(input)
    }
}

fn resolve_key_utf8(key: &str, transformer: &XKBTransformer) -> Result<(Key, KeyModifierFlags)> {
    let mut seq = transformer.utf_to_raw(key.to_string())?;

    // the first entry is the key
    let key = seq.remove(seq.len() - 1);

    let mut flags = KeyModifierFlags::new();

    // the rest are modifiers we have to collect
    for ev in seq.iter() {
        match ev {
            Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_LEFTALT) } => { flags.alt(); }
            Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_RIGHTALT) } => { flags.right_alt(); }
            Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_LEFTSHIFT) } => { flags.shift(); }
            Key { event_code: EventCode::EV_KEY(EV_KEY::KEY_RIGHTSHIFT) } => { flags.shift(); }
            _ => { unreachable!("unhandled modifier") }
        }
    }

    Ok((key, flags))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key() {
        assert_eq!(key("d"), nom_ok((
            Key::from_str("KEY_D").unwrap(),
            KeyModifierFlags::new()
        )));

        assert_eq!(key("btn_forward"), nom_ok((
            Key::from_str("BTN_FORWARD").unwrap(),
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
            Key::from_str("semicolon").unwrap(),
            KeyModifierFlags::new().tap_mut(|x| x.shift())
        )));

        assert_eq!(key_utf(Some(&t))("^"), nom_ok((
            Key::from_str("6").unwrap(),
            KeyModifierFlags::new().tap_mut(|x| x.shift())
        )));
    }

    #[test]
    fn key_special() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();
        use EV_KEY::*;

        assert_eq!(key_utf(Some(&t))("BACKSPACE"), nom_ok((KEY_BACKSPACE.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("BTN_LEFT"), nom_ok((BTN_LEFT.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("BTN_MIDDLE"), nom_ok((BTN_MIDDLE.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("BTN_RIGHT"), nom_ok((BTN_RIGHT.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("CAPSLOCK"), nom_ok((KEY_CAPSLOCK.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("DOWN"), nom_ok((KEY_DOWN.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("ESC"), nom_ok((KEY_ESC.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("LEFT"), nom_ok((KEY_LEFT.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("PAGE_DOWN"), nom_ok((KEY_PAGEDOWN.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("PAGE_UP"), nom_ok((KEY_PAGEUP.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("RIGHT"), nom_ok((KEY_RIGHT.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("SHIFT"), nom_ok((KEY_LEFTSHIFT.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("SPACE"), nom_ok((KEY_SPACE.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("TAB"), nom_ok((KEY_TAB.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("UP"), nom_ok((KEY_UP.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("keypad_1"), nom_ok((KEY_KP1.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("kp1"), nom_ok((KEY_KP1.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("KEYPAD_9"), nom_ok((KEY_KP9.into(), KeyModifierFlags::new())));
        assert_eq!(key_utf(Some(&t))("KP9"), nom_ok((KEY_KP9.into(), KeyModifierFlags::new())));

        assert_eq!(key_utf(Some(&t))("F11"), nom_ok((KEY_F11.into(), KeyModifierFlags::new())));
    }

    #[test]
    fn invalid_key_multiple_chars() {
        let t = XKBTransformer::new("pc105", "us", None, None).unwrap();

        // assert_eq!(key_utf(Some(&t))("ab"), nom_err("ab"));
        assert_nom_err(key_utf(Some(&t))("abc"), "abc");
    }
}