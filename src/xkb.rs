use std::collections::hash_map::Entry;
use std::collections::HashMap;

use evdev_rs::enums::{EventCode, int_to_ev_key};
use itertools::Itertools;
use xkbcommon::xkb;
use xkbcommon::xkb::Keycode;
use xkeysym::Keysym;

use crate::{encoding, Key};

pub struct UTFToRawInputTransformer {
    key_map: HashMap<Keysym, Vec<u32>>,
}

impl UTFToRawInputTransformer {
    pub fn new(model: Option<&str>, layout: Option<&str>, variant: Option<&str>, options: Option<String>) -> Self {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "evdev", // rules
            model.unwrap_or("pc105"),
            layout.unwrap_or("us"),
            variant.unwrap_or_default(),
            options,
            // Some("terminate:ctrl_alt_bksp".to_string()), // options
            xkb::COMPILE_NO_FLAGS,
        ).unwrap();
        let mut state = xkb::State::new(&keymap);

        let mut key_map: HashMap<Keysym, Vec<u32>> = HashMap::new();

        let mods = [
            Keycode::new(50), // left shift
            Keycode::new(108), // right alt
        ];

        keymap.key_for_each(|_, code| {
            let from_keysym = parse_keycode(&state, code);

            if from_keysym.raw() == 0 { return; }

            // backspace corrupts state, skip it
            if code.raw() == 22 { return; }

            for mods in mods.iter().powerset() {
                let mut from = vec![code.raw()];

                for &keycode in mods.iter() {
                    state.update_key(*keycode, xkb::KeyDirection::Down);
                    from.insert(0, keycode.raw());
                }

                let to_keysym = parse_keycode(&state, code);
                if to_keysym.raw() == 0 { return; }

                match key_map.entry(to_keysym) {
                    Entry::Occupied(mut entry) => { if from.len() < entry.get().len() { entry.insert(from); }; }
                    Entry::Vacant(entry) => { entry.insert(from); }
                }

                for &keycode in mods.iter() {
                    state.update_key(*keycode, xkb::KeyDirection::Up);
                }
            }
        });

        Self { key_map }
    }

    pub fn utf_to_raw(&self, key: String) -> Option<Vec<Key>> {
        let encoded = key.chars().next().unwrap() as u32;
        let keysym = encoding::xkb_utf32_to_keysym(encoded);

        self.key_map.get(&keysym).and_then(|keysyms| {
            keysyms.iter()
                .map(|&x|
                    int_to_ev_key(x - 8)
                        .and_then(|x| Some(Key { event_code: EventCode::EV_KEY(x) }))
                )
                .collect::<Option<Vec<Key>>>()
        })
    }
}

fn parse_keycode(state: &xkb::State, keycode: xkb::Keycode) -> Keysym {
    let ucs = state.key_get_utf32(keycode);
    if ucs == 0 {
        state.key_get_one_sym(keycode)
    } else {
        encoding::xkb_utf32_to_keysym(ucs)
    }
}