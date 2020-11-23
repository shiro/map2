use crate::{input_event, State, TAB_DOWN, TAB_REPEAT, TAB_UP, ALT_UP, SHIFT_UP, META_UP, SYN, ALT_DOWN, SHIFT_DOWN, META_DOWN, CAPSLOCK_DOWN, CAPSLOCK_REPEAT, CAPSLOCK_UP, equal, is_modifier_down, print_event, LEFTCTRL_DOWN, ESC_DOWN, ESC_UP, H_DOWN, J_DOWN, K_DOWN, L_DOWN, LEFTCTRL_UP};
use std::{thread, time};

pub fn caps_mod(ev: &input_event, state: &mut State) -> bool {
    if state.capslock_is_down {
        // capslock repeat
        if crate::equal(&ev, &CAPSLOCK_DOWN) || crate::equal(&ev, &CAPSLOCK_REPEAT) {
            return true;
        }

        // capslock up
        if crate::equal(&ev, &CAPSLOCK_UP) {
            state.capslock_is_down = false;
            print_event(&LEFTCTRL_UP);

            // return if up event is ignored
            if crate::ev_ignored(&CAPSLOCK_DOWN, &mut state.ignore_list) {
                crate::unignore_ev(&CAPSLOCK_DOWN, &mut state.ignore_list);
                return true;
            }

            crate::print_event(&ESC_DOWN);
            crate::print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            crate::print_event(&ESC_UP);
            return true;
        }

        // capslock + [key down]
        if ev.value == 1 {
            crate::ignore_ev(&CAPSLOCK_DOWN, &mut state.ignore_list);

            // navigation keys (h, j, k, l)
            // only bind to capslock + directional keys, no modifiers
            if !state.tab_is_down &&
                !is_modifier_down(state) && (
                equal(ev, &H_DOWN) ||
                    equal(ev, &J_DOWN) ||
                    equal(ev, &K_DOWN) ||
                    equal(ev, &L_DOWN)) {
                print_event(&META_DOWN);
                print_event(&ALT_DOWN);
                print_event(&LEFTCTRL_DOWN);
                print_event(&SHIFT_DOWN);
                print_event(ev);

                print_event(&SYN);
                thread::sleep(time::Duration::from_micros(20000));

                print_event(&META_UP);
                print_event(&ALT_UP);
                print_event(&LEFTCTRL_UP);
                print_event(&SHIFT_UP);

                return true;
            }

            return false;
        }
    } else if equal(ev, &CAPSLOCK_DOWN) && !is_modifier_down(state) {
        state.capslock_is_down = true;
        print_event(&LEFTCTRL_DOWN);
        return true;
    } else if equal(ev, &CAPSLOCK_DOWN) { // handle modifier + caps_lock down
        return true;
    } else if equal(ev, &CAPSLOCK_UP) { // handle modifier + caps_lock up
        print_event(&ESC_DOWN);
        print_event(&SYN);
        thread::sleep(time::Duration::from_micros(20000));
        print_event(&ESC_UP);
        return true;
    }
    false
}
