use std::{thread, time};

use crate::{LEFTALT_DOWN, LEFTALT_REPEAT, LEFTALT_UP, ARROW_DOWN_DOWN, ARROW_DOWN_REPEAT, ARROW_DOWN_UP, ARROW_LEFT_DOWN, ARROW_LEFT_REPEAT, ARROW_LEFT_UP, ARROW_RIGHT_DOWN, ARROW_RIGHT_REPEAT, ARROW_RIGHT_UP, ARROW_UP_DOWN, ARROW_UP_REPEAT, ARROW_UP_UP, CAPSLOCK_DOWN, CAPSLOCK_REPEAT, CAPSLOCK_UP, equal, ESC_DOWN, ESC_UP, ev_ignored, H_DOWN, H_REPEAT, H_UP, ignore_ev, input_event, is_modifier_down, J_DOWN, J_REPEAT, J_UP, K_DOWN, K_REPEAT, K_UP, L_DOWN, L_REPEAT, L_UP, LEFTCTRL_DOWN, LEFTCTRL_UP, META_DOWN, META_UP, print_event, SHIFT_DOWN, SHIFT_UP, State, SYN, TAB_DOWN, TAB_REPEAT, TAB_UP, unignore_ev, log_msg, WHEEL};

pub fn leftalt_mod(ev: &input_event, state: &mut State) -> bool {
    if state.leftalt_is_down {
        if equal(ev, &LEFTALT_DOWN) ||
            equal(ev, &LEFTALT_REPEAT) {
            return true;
        }

        if equal(ev, &LEFTALT_UP) {
            state.leftalt_is_down = false;
            if ev_ignored(&LEFTALT_DOWN, &mut state.ignore_list) {
                unignore_ev(&LEFTALT_DOWN, &mut state.ignore_list);
                print_event(&LEFTALT_UP);
                return true;
            }
            print_event(&LEFTALT_DOWN);
            print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&LEFTALT_UP);
            return true;
        }

        // pressed alt + [KEY]
        ignore_ev(&LEFTALT_DOWN, &mut state.ignore_list);

        let mut mapped_key: Option<&input_event> = None;

        if equal(ev, &H_DOWN) {
            mapped_key = Some(&ARROW_LEFT_DOWN);
        } else if equal(ev, &H_UP) {
            mapped_key = Some(&ARROW_LEFT_UP);
        } else if equal(ev, &H_REPEAT) {
            mapped_key = Some(&ARROW_LEFT_REPEAT);
        } else if equal(ev, &J_DOWN) {
            mapped_key = Some(&ARROW_DOWN_DOWN);
        } else if equal(ev, &J_UP) {
            mapped_key = Some(&ARROW_DOWN_UP);
        } else if equal(ev, &J_REPEAT) {
            mapped_key = Some(&ARROW_DOWN_REPEAT);
        } else if equal(ev, &K_DOWN) {
            mapped_key = Some(&ARROW_UP_DOWN);
        } else if equal(ev, &K_UP) {
            mapped_key = Some(&ARROW_UP_UP);
        } else if equal(ev, &K_REPEAT) {
            mapped_key = Some(&ARROW_UP_REPEAT);
        } else if equal(ev, &L_DOWN) {
            mapped_key = Some(&ARROW_RIGHT_DOWN);
        } else if equal(ev, &L_REPEAT) {
            mapped_key = Some(&ARROW_RIGHT_REPEAT);
        } else if equal(ev, &L_UP) {
            mapped_key = Some(&ARROW_RIGHT_UP);
        }


        if let Some(new_ev) = mapped_key {
            print_event(new_ev);
            return true;
        }

        print_event(&LEFTALT_DOWN);
        print_event(&SYN);
        thread::sleep(time::Duration::from_micros(20000));
    } else if equal(ev, &LEFTALT_DOWN) {
        if state.shift_is_down {
            print_event(&LEFTALT_DOWN);

            return true;
        }

        state.leftalt_is_down = true;
        return true;
    }
    false
}
