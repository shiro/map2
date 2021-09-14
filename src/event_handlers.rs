use crate::*;
use messaging::*;
use crate::cli::Configuration;
use crate::python::*;
use pyo3::Python;

pub(crate) fn update_modifiers(state: &mut State, action: &KeyAction) {
    // let ignore_list = &mut state.ignore_list;

    // TODO find a way to do this with a single accessor function
    let pairs: [(Key, fn(&KeyModifierState) -> bool, fn(&mut KeyModifierState) -> &mut bool); 8] = [
        (*KEY_LEFT_CTRL, |s| s.left_ctrl, |s: &mut KeyModifierState| &mut s.left_ctrl),
        (*KEY_RIGHT_CTRL, |s| s.right_ctrl, |s: &mut KeyModifierState| &mut s.right_ctrl),
        (*KEY_LEFT_ALT, |s| s.left_alt, |s: &mut KeyModifierState| &mut s.left_alt),
        (*KEY_RIGHT_ALT, |s| s.right_alt, |s: &mut KeyModifierState| &mut s.right_alt),
        (*KEY_LEFT_SHIFT, |s| s.left_shift, |s: &mut KeyModifierState| &mut s.left_shift),
        (*KEY_RIGHT_SHIFT, |s| s.right_shift, |s: &mut KeyModifierState| &mut s.right_shift),
        (*KEY_LEFT_META, |s| s.left_meta, |s: &mut KeyModifierState| &mut s.left_meta),
        (*KEY_RIGHT_META, |s| s.right_meta, |s: &mut KeyModifierState| &mut s.right_meta),
    ];

    for (key, is_modifier_down, modifier_mut) in pairs.iter() {
        if action.key.event_code == key.event_code && action.value == TYPE_DOWN && !is_modifier_down(&*state.modifiers) {
            let mut new_modifiers = state.modifiers.deref().clone();
            *modifier_mut(&mut new_modifiers) = true;
            state.modifiers = Arc::new(new_modifiers);
            return;
        } else if action.key.event_code == key.event_code && action.value == TYPE_UP {
            let mut new_modifiers = state.modifiers.deref().clone();
            *modifier_mut(&mut new_modifiers) = false;
            state.modifiers = Arc::new(new_modifiers);
            return;
            // TODO re-implement eating or throw it out completely
            // if ignore_list.is_ignored(&KeyAction::new(*key, TYPE_UP)) {
            //     ignore_list.unignore(&KeyAction::new(*key, TYPE_UP));
            //     return;
            // }
        }
    };
}

pub async fn handle_stdin_ev(
    mut state: &mut State,
    ev: InputEvent,
    mappings: &Mappings,
    ev_writer_tx: &mut mpsc::Sender<InputEvent>,
    // modifier_state: &KeyModifierState,
    // message_tx: &mut ExecutionMessageSender,
    // window_cycle_token: &usize,
    // configuration: &Configuration,
) -> Result<()> {
    // if configuration.verbosity >= 3 {
    //     logging::print_debug(format!("input event: {}", logging::print_input_event(&ev)));
    // }

    match ev.event_code {
        EventCode::EV_KEY(_) => {}
        _ => {
            ev_writer_tx.send(ev).await.unwrap();
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
                            ev_writer_tx.send(ev).await.unwrap();
                            ev_writer_tx.send(SYN_REPORT.clone()).await.unwrap();
                        }
                        RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                            let actual_state = &state.modifiers;

                            // takes into account the actual state of a modifier and decides whether to release/restore it or not
                            let release_or_restore_modifier = |is_actual_down: &bool, key: &Key| {
                                if *to_type == 1 { // restore mods if actual mod is still pressed
                                    if *is_actual_down {
                                        // TODO await this once async closures are stable
                                        futures::executor::block_on(ev_writer_tx.send(
                                            KeyAction { key: *key, value: *to_type }.to_input_ev()
                                        )).unwrap();
                                    }
                                } else { // release mods if actual mod is still pressed (prob. always true since it was necessary to trigger the mapping)
                                    if *is_actual_down {
                                        futures::executor::block_on(ev_writer_tx.send(
                                            KeyAction { key: *key, value: *to_type }.to_input_ev()
                                        )).unwrap();
                                    }
                                }
                            };

                            if from_flags.ctrl && !to_flags.ctrl {
                                release_or_restore_modifier(&actual_state.left_ctrl, &*KEY_LEFT_CTRL);
                                release_or_restore_modifier(&actual_state.right_ctrl, &*KEY_RIGHT_CTRL);
                            }
                            if from_flags.shift && !to_flags.shift {
                                release_or_restore_modifier(&actual_state.left_shift, &*KEY_LEFT_SHIFT);
                                release_or_restore_modifier(&actual_state.right_shift, &*KEY_RIGHT_SHIFT);
                            }
                            if from_flags.alt && !to_flags.alt {
                                release_or_restore_modifier(&actual_state.left_alt, &*KEY_LEFT_ALT);
                                release_or_restore_modifier(&actual_state.right_alt, &*KEY_RIGHT_ALT);
                            }
                            if from_flags.meta && !to_flags.meta {
                                release_or_restore_modifier(&actual_state.left_meta, &*KEY_LEFT_META);
                                release_or_restore_modifier(&actual_state.right_meta, &*KEY_RIGHT_META);
                            }

                            // TODO eat keys we just released, un-eat keys we just restored
                        }
                    }
                }
            }
            RuntimeAction::PythonCallback(callback_object) => {
                // use std::time::Instant;
                // let now = Instant::now();
                let gil = Python::acquire_gil();
                let py = gil.python();
                callback_object.call(py, (), None);
                // let elapsed = now.elapsed();
                // println!("Elapsed: {:.2?}", elapsed);
            }
            RuntimeAction::NOP => {}
        }

        return Ok(());
    }

    update_modifiers(&mut state, &KeyAction::from_input_ev(&ev));

    ev_writer_tx.send(ev).await.unwrap();

    Ok(())
}


pub async fn handle_control_message(
    // out: &mut impl Write,
    current_token: usize,
    msg: ControlMessage,
    state: &mut State,
    mappings: &mut Mappings,
    // window_change_handlers: &mut Vec<(Block, GuardedVarMap)>,
) {
    match msg {
        // ExecutionMessage::EatEv(action) => {
        //     state.ignore_list.ignore(&action);
        // }
        ControlMessage::AddMapping(from, to) => {
            // if token == current_token {
            mappings.insert(from, to);
            // }
        }
        // ExecutionMessage::GetFocusedWindowInfo(tx) => {
        //     tx.send(state.active_window.clone()).await.unwrap();
        // }
        // ExecutionMessage::RegisterWindowChangeCallback(block, var_map) => {
        //     window_change_handlers.push((block, var_map));
        // }
        // ExecutionMessage::Write(message) => {
        //     out.write(message.as_ref()).unwrap();
        // }
        // ExecutionMessage::UpdateModifiers(action) => {
        //     event_handlers::update_modifiers(state, &action);
        // }
        // ExecutionMessage::Exit(exit_code) => { std::process::exit(exit_code) }
        // ExecutionMessage::FatalError(err, exit_code) => {
        //     eprintln!("error: {}", err);
        //     std::process::exit(exit_code)
        // }
    }
}


pub fn handle_active_window_change(ev_writer_tx: &mut mpsc::Sender<InputEvent>, message_tx: &mut ExecutionMessageSender,
                                   window_cycle_token: usize, window_change_handlers: &mut Vec<(Block, GuardedVarMap)>) {
    for (handler, var_map) in window_change_handlers {
        let mut message_tx = message_tx.clone();
        let ev_writer_tx = ev_writer_tx.clone();
        let handler = handler.clone();
        let mut var_map = var_map.clone();

        task::spawn(async move {
            eval_block(&handler,
                       &mut var_map,
                       &mut Ambient {
                           ev_writer_tx,
                           message_tx: Some(&mut message_tx),
                           window_cycle_token,
                           modifier_state: &KeyModifierState::new(),
                       },
            ).await;
        });
    }
}