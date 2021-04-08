use crate::*;
use crate::{KeyAction};
use std::{thread, time};
// use crate::KeySequenceItem::KeyAction;

pub fn tab_mod(ev: &crate::input_event, state: &mut State) -> bool {
    if state.tab_is_down {
        // tab repeat
        if ev == &TAB_DOWN || ev == &TAB_REPEAT {
            return true;
        }

        // tab up
        if crate::equal(&ev, &TAB_UP) {
            state.tab_is_down = false;

            // tab up was handled before, just release all mods
            if state.ignore_list.is_ignored(&KeyAction::new(TAB, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(TAB, TYPE_DOWN));
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
            state.ignore_list.ignore(&KeyAction::new(TAB, TYPE_DOWN));
            crate::print_event(&LEFTALT_DOWN);
            crate::print_event(&SHIFT_DOWN);
            crate::print_event(&META_DOWN);
            crate::print_event(&SYN);
            thread::sleep(time::Duration::from_micros(20000));
            print_event(&ev);

            print_event(&LEFTALT_UP);
            print_event(&SHIFT_UP);
            print_event(&META_UP);

            return true;
        }
    } else if ev == &TAB_DOWN {
        state.tab_is_down = true;
        return true;
    }

    false
}
