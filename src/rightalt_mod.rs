use std::{thread, time};
use crate::*;

pub fn rightalt_mod(ev: &crate::input_event, state: &mut State) -> bool {
    if state.right_alt_is_down {
        if ev == &RIGHTALT_DOWN ||
            ev == &RIGHTALT_REPEAT {
            return true;
        }

        if ev == &RIGHTALT_UP {
            state.right_alt_is_down = false;
            if state.ignore_list.is_ignored(&KeyAction::new(RIGHT_ALT, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(RIGHT_ALT, TYPE_DOWN));
                print_event(&RIGHTALT_UP);
                print_event(&META_UP);
                return true;
            }

            print_event(&RIGHTALT_DOWN);
            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&RIGHTALT_UP);
            return true;
        }

        // pressed right_alt + [KEY]
        if ev.code > 0 {
            // mod only specific keys
            if vec![KEY_H as u16, KEY_J as u16, KEY_K as u16, KEY_L as u16].contains(&ev.code) {
                print_event(&LEFTALT_DOWN);
                print_event(&META_DOWN);

                // ignore right alt release
                state.ignore_list.unignore(&KeyAction::new(RIGHT_ALT, TYPE_DOWN));
            } else {
                print_event(&RIGHTALT_DOWN);
            }

            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(ev);

            return true;
        }
    } else if ev == &RIGHTALT_DOWN {
        state.right_alt_is_down = true;
        return true;
    }
    false
}
