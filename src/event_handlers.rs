use crate::*;
use crate::device::virtual_output_device::VirtualOutputDevice;
use crate::mapper::*;

pub(crate) fn update_modifiers(state: &mut State, action: &KeyAction) -> bool {
    // TODO find a way to do this with a single accessor function
    let pairs: [(Key, fn(&KeyModifierState) -> bool, fn(&mut KeyModifierState) -> &mut bool); 8] = [
        (KEY_LEFTCTRL.into(), |s| s.left_ctrl, |s: &mut KeyModifierState| &mut s.left_ctrl),
        (KEY_RIGHTCTRL.into(), |s| s.right_ctrl, |s: &mut KeyModifierState| &mut s.right_ctrl),
        (KEY_LEFTALT.into(), |s| s.left_alt, |s: &mut KeyModifierState| &mut s.left_alt),
        (KEY_RIGHTALT.into(), |s| s.right_alt, |s: &mut KeyModifierState| &mut s.right_alt),
        (KEY_LEFTSHIFT.into(), |s| s.left_shift, |s: &mut KeyModifierState| &mut s.left_shift),
        (KEY_RIGHTSHIFT.into(), |s| s.right_shift, |s: &mut KeyModifierState| &mut s.right_shift),
        (KEY_LEFTMETA.into(), |s| s.left_meta, |s: &mut KeyModifierState| &mut s.left_meta),
        (KEY_RIGHTMETA.into(), |s| s.right_meta, |s: &mut KeyModifierState| &mut s.right_meta),
    ];

    for (key, is_modifier_down, modifier_mut) in pairs.iter() {
        if action.key.event_code == key.event_code && action.value == TYPE_DOWN && !is_modifier_down(&*state.modifiers) {
            let mut new_modifiers = state.modifiers.deref().clone();
            *modifier_mut(&mut new_modifiers) = true;
            state.modifiers = Arc::new(new_modifiers);
            return true;
        } else if action.key.event_code == key.event_code && action.value == TYPE_UP {
            let mut new_modifiers = state.modifiers.deref().clone();
            *modifier_mut(&mut new_modifiers) = false;
            state.modifiers = Arc::new(new_modifiers);
            return true;
            // TODO re-implement eating or throw it out completely
            // if ignore_list.is_ignored(&KeyAction::new(*key, TYPE_UP)) {
            //     ignore_list.unignore(&KeyAction::new(*key, TYPE_UP));
            //     return;
            // }
        } else if action.value == TYPE_REPEAT {
            return true;
        }
    };
    false
}

fn release_restore_modifiers(state: &mut State, output_device: &mut VirtualOutputDevice, from_flags: &KeyModifierFlags, to_flags: &KeyModifierFlags, to_type: &i32) {
    let actual_state = &state.modifiers;

    // takes into account the actual state of a modifier and decides whether to release/restore it or not
    let mut release_or_restore_modifier = |is_actual_down: &bool, key: &Key| {
        if *to_type == 1 { // restore mods if actual mod is still pressed
            if *is_actual_down {
                output_device.send(
                    &KeyAction { key: *key, value: *to_type }.to_input_ev()
                ).unwrap();
            }
        } else { // release mods if actual mod is still pressed (prob. always true since it was necessary to trigger the mapping)
            if *is_actual_down {
                output_device.send(
                    &KeyAction { key: *key, value: *to_type }.to_input_ev()
                ).unwrap();
            }
        }
    };

    if from_flags.ctrl && !to_flags.ctrl {
        release_or_restore_modifier(&actual_state.left_ctrl, &KEY_LEFTCTRL.into());
        release_or_restore_modifier(&actual_state.right_ctrl, &KEY_RIGHTCTRL.into());
    }
    if from_flags.shift && !to_flags.shift {
        release_or_restore_modifier(&actual_state.left_shift, &KEY_LEFTSHIFT.into());
        release_or_restore_modifier(&actual_state.right_shift, &KEY_RIGHTSHIFT.into());
    }
    if from_flags.alt && !to_flags.alt {
        release_or_restore_modifier(&actual_state.left_alt, &KEY_LEFTALT.into());
        release_or_restore_modifier(&actual_state.right_alt, &KEY_RIGHTALT.into());
    }
    if from_flags.meta && !to_flags.meta {
        release_or_restore_modifier(&actual_state.left_meta, &KEY_LEFTMETA.into());
        release_or_restore_modifier(&actual_state.right_meta, &KEY_RIGHTMETA.into());
    }

    // TODO eat keys we just released, un-eat keys we just restored
}

pub fn handle_stdin_ev(
    mut state: &mut State,
    ev: EvdevInputEvent,
    mappings: &Mappings,
    output_device: &mut VirtualOutputDevice,
) -> Result<()> {
    match ev.event_code {
        EventCode::EV_KEY(_) => {}
        _ => {
            output_device.send(&ev).unwrap();
            return Ok(());
        }
    }

    let mut from_modifiers = KeyModifierFlags::new();
    from_modifiers.ctrl = state.modifiers.is_ctrl();
    from_modifiers.alt = state.modifiers.is_alt();
    from_modifiers.shift = state.modifiers.is_shift();
    from_modifiers.meta = state.modifiers.is_meta();

    let from_key_action = KeyActionWithMods {
        key: Key { event_code: ev.event_code },
        value: ev.value,
        modifiers: from_modifiers,
    };

    if let Some(runtime_action) = mappings.get(&from_key_action) {
        match runtime_action {
            RuntimeAction::ActionSequence(seq) => {
                for action in seq {
                    match action {
                        RuntimeKeyAction::KeyAction(key_action) => {
                            let ev = key_action.to_input_ev();
                            output_device.send(&ev).unwrap();
                            output_device.send(&SYN_REPORT).unwrap();
                        }
                        RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                            release_restore_modifiers(state, output_device, from_flags, to_flags, to_type);
                        }
                    }
                }
            }
            RuntimeAction::PythonCallback(from_modifiers, callback_object) => {
                // always release all trigger mods before running the callback
                release_restore_modifiers(state, output_device, from_modifiers, &KeyModifierFlags::new(), &TYPE_UP);

                EVENT_LOOP.lock().unwrap().execute(callback_object.clone(), None);
            }
            RuntimeAction::NOP => {}
        }

        return Ok(());
    }

    update_modifiers(&mut state, &KeyAction::from_input_ev(&ev));

    output_device.send(&ev).unwrap();

    Ok(())
}