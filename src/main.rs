use map2::*;
use map2::messaging::*;
use std::ops::Deref;
use std::thread;

#[tokio::main]
async fn main() -> Result<()> {
    let mut configuration = parse_cli()?;

    let (window_ev_tx, mut window_ev_rx) = tokio::sync::mpsc::channel(128);
    let (mut message_tx, mut message_rx) = tokio::sync::mpsc::channel(128);

    // x11 thread
    tokio::spawn(async move {
        let x11_state = Arc::new(x11_initialize().unwrap());

        loop {
            let x11_state_clone = x11_state.clone();
            let res = task::spawn_blocking(move || {
                x11_test(&x11_state_clone)
            }).await.unwrap();

            if let Ok(Some(val)) = res {
                window_ev_tx.send(val).await.unwrap_or_else(|_| panic!());
            }
        }
    });


    let mut state = State::new();
    let global_scope = mappings::bind_mappings(&mut configuration.script_file);
    let mut window_cycle_token: usize = 0;
    let mut mappings = CompiledKeyMappings::new();

    let (ev_reader_init_tx, ev_reader_init_rx) = oneshot::channel();
    let (ev_writer_tx, mut ev_writer_rx) = mpsc::channel(128);

    // add a small delay if run from TTY so we don't miss 'enter up' which is often released when the device is grabbed
    if atty::is(atty::Stream::Stdout) {
        thread::sleep(time::Duration::from_millis(300));
    }

    // start coroutine
    bind_udev_inputs(&configuration.devices, ev_reader_init_tx, ev_writer_tx).await?;
    let mut ev_reader_tx = ev_reader_init_rx.await?;

    let mut window_change_handlers = vec![];
    {
        let mut message_tx = message_tx.clone();
        let ev_reader_tx = ev_reader_tx.clone();
        task::spawn(async move {
            let mut amb = Ambient {
                ev_writer_tx: ev_reader_tx,
                window_cycle_token,
                message_tx: Some(&mut message_tx),
                modifier_state: KeyModifierState::new(),
            };

            eval_block(&global_scope, &mut GuardedVarMap::new(Mutex::new(VarMap::new(None))), &mut amb).await;
        });
    }

    fn handle_active_window_change(ev_writer_tx: &mut mpsc::Sender<InputEvent>, message_tx: &mut ExecutionMessageSender,
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
                               modifier_state: KeyModifierState::new(),
                           },
                ).await;
            });
        }
    }

    async fn handle_execution_message(current_token: usize, msg: ExecutionMessage, state: &mut State, mappings: &mut CompiledKeyMappings,
                                      window_change_handlers: &mut Vec<(Block, GuardedVarMap)>) {
        match msg {
            ExecutionMessage::EatEv(action) => {
                state.ignore_list.ignore(&action);
            }
            ExecutionMessage::AddMapping(token, from, to, var_map) => {
                if token == current_token {
                    mappings.0.insert(from, Arc::new((to, var_map)));
                }
            }
            ExecutionMessage::GetFocusedWindowInfo(tx) => {
                tx.send(state.active_window.clone()).await.unwrap();
            }
            ExecutionMessage::RegisterWindowChangeCallback(block, var_map) => {
                window_change_handlers.push((block, var_map));
            }
            ExecutionMessage::Exit(exit_code) => { std::process::exit(exit_code) }
            ExecutionMessage::FatalError(err, exit_code) => {
                eprintln!("error: {}", err);
                std::process::exit(exit_code)
            }
        }
    }

    loop {
        tokio::select! {
            Some(window) = window_ev_rx.recv() => {
                state.active_window = Some(window);
                window_cycle_token = window_cycle_token + 1;
                handle_active_window_change(&mut ev_reader_tx,
                    &mut message_tx, window_cycle_token, &mut window_change_handlers);
            }
            Some(ev) = ev_writer_rx.recv() => {
                handle_stdin_ev(&mut state, ev, &mut mappings,
                    &mut ev_reader_tx, &mut message_tx, window_cycle_token).await.unwrap();
            }
            Some(msg) = message_rx.recv() => {
                handle_execution_message(window_cycle_token, msg, &mut state, &mut mappings, &mut window_change_handlers).await;
            }
            else => { break }
        }
    }

    Ok(())
}


fn update_modifiers(state: &mut State, ev: &InputEvent) {
    let ignore_list = &mut state.ignore_list;
    vec![
        (*KEY_LEFT_CTRL, &mut state.modifiers.left_ctrl),
        (*KEY_RIGHT_CTRL, &mut state.modifiers.right_ctrl),
        (*KEY_LEFT_ALT, &mut state.modifiers.left_alt),
        (*KEY_RIGHT_ALT, &mut state.modifiers.right_alt),
        (*KEY_LEFT_SHIFT, &mut state.modifiers.left_shift),
        (*KEY_RIGHT_SHIFT, &mut state.modifiers.right_shift),
        (*KEY_LEFT_META, &mut state.modifiers.left_meta),
        (*KEY_RIGHT_META, &mut state.modifiers.right_meta),
    ]
        .iter_mut()
        .for_each(|(a, b)| {
            if ev.value == TYPE_DOWN && ev.event_code == a.event_code {
                **b = true;
            } else if ev.value == TYPE_UP && ev.event_code == a.event_code {
                **b = false;
                if ignore_list.is_ignored(&KeyAction::new(*a, TYPE_UP)) {
                    ignore_list.unignore(&KeyAction::new(*a, TYPE_UP));
                    return;
                }
            }
        });
}

async fn handle_stdin_ev(mut state: &mut State, ev: InputEvent,
                         mappings: &mut CompiledKeyMappings,
                         ev_writer: &mut mpsc::Sender<InputEvent>, message_tx: &mut ExecutionMessageSender, window_cycle_token: usize) -> Result<()> {
    match ev.event_code {
        EventCode::EV_KEY(_) => {}
        _ => {
            ev_writer.send(ev).await.unwrap();
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

    update_modifiers(&mut state, &ev);

    if let Some(block) = mappings.0.get(&from_key_action) {
        let block = block.clone();
        let mut message_tx = message_tx.clone();
        let ev_writer = ev_writer.clone();
        let modifier_state = state.modifiers.clone();
        task::spawn(async move {
            let (block, var_map) = block.deref();
            let mut amb = Ambient { ev_writer_tx: ev_writer, message_tx: Some(&mut message_tx), window_cycle_token, modifier_state };

            eval_block(&block, &var_map, &mut amb).await;
        });
        return Ok(());
    }

    ev_writer.send(ev).await.unwrap();

    Ok(())
}
