use crate::*;
use std::{thread, time};

pub fn caps_mod(ev: &input_event, state: &mut State) -> bool {
    if state.capslock_is_down {
        if *ev == INPUT_EV_CAPSLOCK.down || *ev == INPUT_EV_CAPSLOCK.repeat {
            return true;
        }

        if *ev == INPUT_EV_CAPSLOCK.up {
            state.capslock_is_down = false;
            print_event(&INPUT_EV_LEFTCTRL.up);

            if state.ignore_list.is_ignored(&KeyAction::new(KEY_CAPSLOCK, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(KEY_CAPSLOCK, TYPE_DOWN));
                return true;
            }

            crate::print_event(&INPUT_EV_ESC.down);
            crate::print_event(&INPUT_EV_SYN);
            thread::sleep(time::Duration::from_micros(20000));
            crate::print_event(&INPUT_EV_ESC.up);
            return true;
        }

        // capslock + [key down]
        if ev.value == 1 {
            state.ignore_list.ignore(&KeyAction::new(KEY_CAPSLOCK, TYPE_DOWN));

            // navigation keys (h, j, k, l)
            // only bind to capslock + directional keys, no modifiers
            if !state.tab_is_down &&
                !state.is_any_modifier_down() && (
                *ev == INPUT_EV_H.down ||
                    *ev == INPUT_EV_J.down ||
                    *ev == INPUT_EV_K.down ||
                    *ev == INPUT_EV_L.down) {
                print_event(&INPUT_EV_LEFTMETA.down);
                print_event(&INPUT_EV_LEFTALT.down);
                print_event(&INPUT_EV_LEFTCTRL.down);
                print_event(&INPUT_EV_SHIFT.down);
                print_event(ev);

                print_event(&INPUT_EV_SYN);
                thread::sleep(time::Duration::from_micros(20000));

                print_event(&INPUT_EV_LEFTMETA.up);
                print_event(&INPUT_EV_LEFTALT.up);
                print_event(&INPUT_EV_LEFTCTRL.up);
                print_event(&INPUT_EV_SHIFT.up);

                return true;
            }

            return false;
        }
    } else if *ev == INPUT_EV_CAPSLOCK.down && !state.is_any_modifier_down() {
        state.capslock_is_down = true;
        print_event(&INPUT_EV_LEFTCTRL.down);
        return true;
    } else if *ev == INPUT_EV_CAPSLOCK.down { // handle modifier + caps_lock down
        return true;
    } else if *ev == INPUT_EV_CAPSLOCK.up { // handle modifier + caps_lock up
        print_event(&INPUT_EV_ESC.down);
        print_event(&INPUT_EV_SYN);
        thread::sleep(time::Duration::from_micros(20000));
        print_event(&INPUT_EV_ESC.up);
        return true;
    }
    false
}
