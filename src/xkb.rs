use std::collections::hash_map::Entry;
use std::collections::HashMap;

use evdev_rs::enums::{EV_KEY, EventCode, int_to_ev_key};
use itertools::Itertools;
use xkbcommon::xkb;
use xkbcommon::xkb::Keycode;
use xkeysym::Keysym;

use crate::*;
use crate::{encoding, Key};

#[derive(Clone)]
pub struct UTFToRawInputTransformer {
    utf_to_raw_map: HashMap<Keysym, Vec<u32>>,
    raw_to_utf_map: HashMap<(EV_KEY, KeyModifierState), String>,
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
        let mut xkb_state = xkb::State::new(&keymap);

        let mut utf_to_raw_map: HashMap<Keysym, Vec<u32>> = HashMap::new();
        let mut raw_to_utf_map = HashMap::new();

        let mods = [
            ("LEFT_SHIFT", Keycode::new(50)),
            ("RIGHT_SHIFT", Keycode::new(62)),
            ("LEFT_ALT", Keycode::new(64)),
            ("RIGHT_ALT", Keycode::new(108)),
        ];

        keymap.key_for_each(|_, code| {
            let from_keysym = parse_keycode(&xkb_state, code);

            if from_keysym.raw() == 0 { return; }

            // backspace corrupts state, skip it
            if code.raw() == 22 { return; }

            for mods in mods.iter().powerset() {
                let mut from = vec![code.raw()];

                for &(_, keycode) in mods.iter() {
                    xkb_state.update_key(*keycode, xkb::KeyDirection::Down);
                    from.insert(0, keycode.raw());
                }

                let to_keysym = parse_keycode(&xkb_state, code);
                if to_keysym.raw() == 0 { return; }

                match utf_to_raw_map.entry(to_keysym) {
                    Entry::Occupied(mut entry) => { if from.len() < entry.get().len() { entry.insert(from); }; }
                    Entry::Vacant(entry) => { entry.insert(from); }
                }

                if let Some(ev_key) = int_to_ev_key(code.raw() - 8) {
                    if let Some(name) = char::from_u32(xkb_state.key_get_utf32(code)) {
                        let mut state = KeyModifierState::new();

                        for &(name, _) in mods.iter() {
                            match *name {
                                "LEFT_SHIFT" => { state.left_shift = true; }
                                "RIGHT_SHIFT" => { state.right_shift = true; }
                                "LEFT_ALT" => { state.left_alt = true; }
                                "RIGHT_ALT" => { state.right_alt = true; }
                                _ => {}
                            }
                        }

                        match raw_to_utf_map.entry((ev_key, state)) {
                            Entry::Vacant(entry) => { entry.insert(name.to_string()); }
                            _ => {}
                        }
                    }
                }

                for &(_, keycode) in mods.iter() {
                    xkb_state.update_key(*keycode, xkb::KeyDirection::Up);
                }
            }
        });

        Self {
            utf_to_raw_map,
            raw_to_utf_map,
        }
    }

    pub fn utf_to_raw(&self, key: String) -> Result<Vec<Key>> {
        let mut it = key.chars().into_iter();
        let encoded = it.next().unwrap() as u32;

        if it.next().is_some() {
            return Err(anyhow!("received more than 1 UTF character"));
        }

        let keysym = encoding::xkb_utf32_to_keysym(encoded);

        self.utf_to_raw_map.get(&keysym).and_then(|keysyms| {
            keysyms.iter()
                .map(|&x|
                    int_to_ev_key(x - 8)
                        .and_then(|x| Some(Key { event_code: EventCode::EV_KEY(x) }))
                )
                .collect::<Option<Vec<Key>>>()
        }).ok_or_else(|| anyhow!("failed to convert utf to raw"))
    }

    pub fn raw_to_utf(&self, key: &EV_KEY, state: &KeyModifierState) -> Option<String> {
        self.raw_to_utf_map.get(&(*key, *state)).cloned()
    }
}

fn parse_keycode(state: &xkb::State, keycode: Keycode) -> Keysym {
    let ucs = state.key_get_utf32(keycode);
    if ucs == 0 {
        state.key_get_one_sym(keycode)
    } else {
        encoding::xkb_utf32_to_keysym(ucs)
    }
}