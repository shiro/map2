use std::{thread, time};
use crate::*;

pub fn leftalt_mod(ev: &crate::input_event, state: &mut State) -> bool {
    if state.leftalt_is_down {
        if ev == &LEFTALT_DOWN ||
            ev == &LEFTALT_REPEAT {
            return true;
        }

        if ev == &LEFTALT_UP {
            state.leftalt_is_down = false;
            if state.ignore_list.is_ignored(&KeyAction::new(LEFT_ALT, TYPE_DOWN)) {
                state.ignore_list.unignore(&KeyAction::new(LEFT_ALT, TYPE_DOWN));
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
        state.ignore_list.ignore(&KeyAction::new(LEFT_ALT, TYPE_DOWN));

        let mut mapped_key: Option<&crate::input_event> = None;

        if ev == &H_DOWN {
            mapped_key = Some(&ARROW_LEFT_DOWN);
        } else if ev == &H_UP {
            mapped_key = Some(&ARROW_LEFT_UP);
        } else if ev == &H_REPEAT {
            mapped_key = Some(&ARROW_LEFT_REPEAT);
        } else if ev == &J_DOWN {
            mapped_key = Some(&ARROW_DOWN_DOWN);
        } else if ev == &J_UP {
            mapped_key = Some(&ARROW_DOWN_UP);
        } else if ev == &J_REPEAT {
            mapped_key = Some(&ARROW_DOWN_REPEAT);
        } else if ev == &K_DOWN {
            mapped_key = Some(&ARROW_UP_DOWN);
        } else if ev == &K_UP {
            mapped_key = Some(&ARROW_UP_UP);
        } else if ev == &K_REPEAT {
            mapped_key = Some(&ARROW_UP_REPEAT);
        } else if ev == &L_DOWN {
            mapped_key = Some(&ARROW_RIGHT_DOWN);
        } else if ev == &L_REPEAT {
            mapped_key = Some(&ARROW_RIGHT_REPEAT);
        } else if ev == &L_UP {
            mapped_key = Some(&ARROW_RIGHT_UP);
        }


        if let Some(new_ev) = mapped_key {
            print_event(new_ev);

            log_msg("key down");
            // print_event(&LEFTALT_UP);
            return true;
        }

        print_event(&LEFTALT_DOWN);
        log_msg("alt down");
        print_event(&SYN);
        thread::sleep(time::Duration::from_micros(20000));

        // return true;
    } else if ev == &LEFTALT_DOWN {
        state.leftalt_is_down = true;
        return true;
    }
    false
}
