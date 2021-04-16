use std::{thread, time};
use crate::*;

pub fn rightalt_mod(ev: &crate::input_event, state: &mut State) -> bool {
    if state.right_alt_is_down {
        if ev == &INPUT_EV_RIGHTALT.down ||
            ev == &INPUT_EV_RIGHTALT.repeat {
            return true;
        }

        if ev == &INPUT_EV_RIGHTALT.up {
            state.right_alt_is_down = false;
            if state.ignore_list.is_ignored(&KeyAction::new(KEY_RIGHT_ALT, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(KEY_RIGHT_ALT, TYPE_DOWN));
                print_event(&INPUT_EV_RIGHTALT.up);
                print_event(&INPUT_EV_META.up);
                return true;
            }

            print_event(&INPUT_EV_RIGHTALT.down);
            print_event(&INPUT_EV_SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&INPUT_EV_RIGHTALT.up);
            return true;
        }

        // pressed right_alt + [KEY]
        if ev.code > 0 {
            // mod only specific keys
            if vec![input_linux_sys::KEY_H as u16, input_linux_sys::KEY_J as u16, input_linux_sys::KEY_K as u16, input_linux_sys::KEY_L as u16]
                .iter().any(|&code| code == ev.code) {
                print_event(&INPUT_EV_LEFTALT.down);
                print_event(&INPUT_EV_META.down);

                // ignore right alt release
                state.ignore_list.unignore(&KeyAction::new(KEY_RIGHT_ALT, TYPE_DOWN));
            } else {
                print_event(&INPUT_EV_RIGHTALT.down);
            }

            print_event(&INPUT_EV_SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(ev);

            return true;
        }
    } else if ev == &INPUT_EV_RIGHTALT.down {
        state.right_alt_is_down = true;
        return true;
    }
    false
}
