use crate::*;
use std::{thread, time};

pub fn caps_mod(ev: &crate::input_event, state: &mut State) -> bool {
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
            if state.ignore_list.is_ignored(&KeyAction::new(CAPSLOCK, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(CAPSLOCK, TYPE_DOWN));
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
            state.ignore_list.ignore(&KeyAction::new(CAPSLOCK, TYPE_DOWN));

            // navigation keys (h, j, k, l)
            // only bind to capslock + directional keys, no modifiers
            if !state.tab_is_down &&
                !is_modifier_down(state) && (
                equal(ev, &H_DOWN) ||
                    equal(ev, &J_DOWN) ||
                    equal(ev, &K_DOWN) ||
                    equal(ev, &L_DOWN)) {
                print_event(&META_DOWN);
                print_event(&LEFTALT_DOWN);
                print_event(&LEFTCTRL_DOWN);
                print_event(&SHIFT_DOWN);
                print_event(ev);

                print_event(&SYN);
                thread::sleep(time::Duration::from_micros(20000));

                print_event(&META_UP);
                print_event(&LEFTALT_UP);
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
