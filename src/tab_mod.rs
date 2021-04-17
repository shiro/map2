use crate::*;
use std::{thread, time};

pub fn tab_mod(ev: &input_event, state: &mut State) -> bool {
    if state.tab_is_down {
        if *ev == INPUT_EV_TAB.down || *ev == INPUT_EV_TAB.repeat {
            return true;
        }

        if *ev == INPUT_EV_TAB.up {
            state.tab_is_down = false;

            if state.ignore_list.is_ignored(&KeyAction::new(KEY_TAB, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(KEY_TAB, TYPE_DOWN));
                return true;
            }

            print_event(&INPUT_EV_TAB.down);
            print_event(&INPUT_EV_SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&INPUT_EV_TAB.up);
            return true;
        }

        // tab + [key down]
        if ev.value == 1 {
            state.ignore_list.ignore(&KeyAction::new(KEY_TAB, TYPE_DOWN));
            print_event(&INPUT_EV_LEFTALT.down);
            print_event(&INPUT_EV_SHIFT.down);
            print_event(&INPUT_EV_LEFTMETA.down);
            print_event(&INPUT_EV_SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&ev);

            print_event(&INPUT_EV_LEFTALT.up);
            print_event(&INPUT_EV_SHIFT.up);
            print_event(&INPUT_EV_LEFTMETA.up);

            return true;
        }
    } else if ev == &INPUT_EV_TAB.down {
        state.tab_is_down = true;
        return true;
    }

    false
}
