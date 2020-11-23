use std::{thread, time};

use crate::{LEFTALT_DOWN, LEFTALT_REPEAT, LEFTALT_UP, ARROW_DOWN_DOWN, ARROW_DOWN_REPEAT, ARROW_DOWN_UP, ARROW_LEFT_DOWN, ARROW_LEFT_REPEAT, ARROW_LEFT_UP, ARROW_RIGHT_DOWN, ARROW_RIGHT_REPEAT, ARROW_RIGHT_UP, ARROW_UP_DOWN, ARROW_UP_REPEAT, ARROW_UP_UP, CAPSLOCK_DOWN, CAPSLOCK_REPEAT, CAPSLOCK_UP, equal, ESC_DOWN, ESC_UP, ev_ignored, H_DOWN, H_REPEAT, H_UP, ignore_ev, input_event, is_modifier_down, J_DOWN, J_REPEAT, J_UP, K_DOWN, K_REPEAT, K_UP, L_DOWN, L_REPEAT, L_UP, LEFTCTRL_DOWN, LEFTCTRL_UP, META_DOWN, META_UP, print_event, SHIFT_DOWN, SHIFT_UP, State, SYN, TAB_DOWN, TAB_REPEAT, TAB_UP, unignore_ev, RIGHTALT_DOWN, RIGHTALT_REPEAT, RIGHTALT_UP};
use input_linux_sys::{KEY_H, KEY_J, KEY_L, KEY_K};

pub fn rightalt_mod(ev: &input_event, state: &mut State) -> bool {
    if state.right_alt_is_down {
        if equal(ev, &RIGHTALT_DOWN) ||
            equal(ev, &RIGHTALT_REPEAT) {
            return true;
        }

        if equal(ev, &RIGHTALT_UP) {
            state.right_alt_is_down = false;
            if ev_ignored(&RIGHTALT_UP, &mut state.ignore_list) {
                unignore_ev(&RIGHTALT_UP, &mut state.ignore_list);
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
                ignore_ev(&RIGHTALT_UP, &mut state.ignore_list);
            } else {
                print_event(&RIGHTALT_DOWN);
            }

            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(ev);

            return true;
        }
    } else if equal(ev, &RIGHTALT_DOWN) {
        state.right_alt_is_down = true;
        return true;
    }
    false
}
