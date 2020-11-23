use crate::{input_event, State, TAB_DOWN, TAB_REPEAT, TAB_UP, LEFTALT_UP, SHIFT_UP, META_UP, SYN, LEFTALT_DOWN, SHIFT_DOWN, META_DOWN};
use std::{thread, time};

pub fn tab_mod(ev: &input_event, state: &mut State) -> bool {
    if state.tab_is_down {
        // tab repeat
        if crate::equal(&ev, &TAB_DOWN) || crate::equal(&ev, &TAB_REPEAT) {
            return true;
        }

        // tab up
        if crate::equal(&ev, &TAB_UP) {
            state.tab_is_down = false;

            // tab up was handled before, just release all mods
            if crate::ev_ignored(&TAB_DOWN, &mut state.ignore_list) {
                crate::unignore_ev(&TAB_DOWN, &mut state.ignore_list);
                crate::print_event(&LEFTALT_UP);
                crate::print_event(&SHIFT_UP);
                crate::print_event(&META_UP);
                return true;
            }

            crate::print_event(&TAB_DOWN);
            crate::print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            crate::print_event(&TAB_UP);
            return true;
        }

        // tab + [key down]
        if ev.value == 1 {
            crate::ignore_ev(&TAB_DOWN, &mut state.ignore_list);
            crate::print_event(&LEFTALT_DOWN);
            crate::print_event(&SHIFT_DOWN);
            crate::print_event(&META_DOWN);
            crate::print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
        }
    } else if crate::equal(&ev, &TAB_DOWN) {
        state.tab_is_down = true;
        return true;
    }

    false
}
