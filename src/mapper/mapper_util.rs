use evdev_rs::enums::EV_KEY;
use tokio::sync::MutexGuard;

use crate::python::*;
use crate::*;

use self::xkb::XKBTransformer;
use crate::event_loop::{args_to_py, PythonArgument};

pub fn hash_path(path: &Vec<uuid::Uuid>) -> u64 {
    use std::hash::Hash;
    use std::hash::Hasher;
    let mut h = std::hash::DefaultHasher::new();
    path.hash(&mut h);
    let path_hash = h.finish();
    path_hash
}

#[derive(Debug, Clone)]
pub enum PythonReturn {
    String(String),
    Bool(bool),
}

pub async fn run_python_handler(
    handler: Arc<PyObject>,
    args: Option<Vec<PythonArgument>>,
    ev: EvdevInputEvent,
    transformer: Arc<XKBTransformer>,
    // next: &HashMap<Uuid, Arc<dyn LinkDst>>,
    next: Vec<Arc<dyn LinkDst>>,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let ret = Python::with_gil(|py| -> Result<()> {
            let asyncio =
                py.import_bound("asyncio").expect("python runtime error: failed to import 'asyncio', is it installed?");

            let is_async_callback: bool = asyncio
                .call_method1("iscoroutinefunction", (handler.deref().bind(py),))
                .expect("python runtime error: 'iscoroutinefunction' lookup failed")
                .extract()
                .expect("python runtime error: 'iscoroutinefunction' call failed");

            if is_async_callback {
                // TODO spawn a task here, run cb
                // EVENT_LOOP.lock().unwrap().execute(&handler, args);
                Ok(())
            } else {
                let args = args_to_py(py, args.unwrap_or(vec![]));
                let ret = handler.call_bound(py, args, None).map_err(|err| anyhow!("{}", err)).and_then(|ret| {
                    if ret.is_none(py) {
                        return Ok(None);
                    }

                    if let Ok(ret) = ret.extract::<String>(py) {
                        return Ok(Some(PythonReturn::String(ret)));
                    }
                    if let Ok(ret) = ret.extract::<bool>(py) {
                        return Ok(Some(PythonReturn::Bool(ret)));
                    }

                    Err(anyhow!("unsupported python return value"))
                })?;

                match ret {
                    Some(PythonReturn::String(ret)) => {
                        let seq = parse_key_sequence(&ret, Some(&transformer))?;

                        for action in seq.to_key_actions() {
                            next.send_all(InputEvent::Raw(action.to_input_ev()));
                        }
                    }
                    Some(PythonReturn::Bool(ret)) if ret => {
                        next.send_all(InputEvent::Raw(ev.clone()));
                    }
                    _ => {}
                };
                Ok(())
            }
        });
        if let Err(err) = ret {
            eprintln!("{err}");
            std::process::exit(1);
        }
    })
    .await?;

    Ok(())
}

pub fn python_callback_args(
    ev: &EventCode,
    modifiers: &KeyModifierFlags,
    value: i32,
    transformer: &XKBTransformer,
) -> Vec<PythonArgument> {
    let (name, format_value) = match ev {
        EventCode::EV_KEY(key) => (
            match key {
                KEY_SPACE => "space".to_string(),
                KEY_TAB => "tab".to_string(),
                KEY_ENTER => "enter".to_string(),
                _ => transformer.raw_to_utf(key, *modifiers).unwrap_or_else(|| {
                    let name = format!("{key:?}").to_string().to_lowercase();
                    name.strip_prefix("key_").unwrap_or(&name).to_string()
                }),
            },
            true,
        ),
        EventCode::EV_REL(ev) => (format!("{ev:?}").to_lowercase(), false),
        EventCode::EV_ABS(ev) => (format!("{ev:?}").to_lowercase(), false),
        val => panic!("got unexpected value: {}", val),
    };

    let value = if format_value {
        PythonArgument::String(
            match value {
                0 => "up",
                1 => "down",
                2 => "repeat",
                _ => unreachable!(),
            }
            .to_string(),
        )
    } else {
        PythonArgument::Number(value)
    };

    vec![PythonArgument::String(name), value]
}

pub fn handle_seq<Next: SubscriberHashmapExt>(seq: &Vec<KeyActionWithMods>, _flags: &KeyModifierFlags, next: &Next) {
    let mut flags = _flags.clone();
    // let mut prev = None;
    for action in seq {
        for action in release_restore_modifiers(&flags, &action.modifiers) {
            let _ = next.send_all(InputEvent::Raw(action.to_input_ev()));
        }

        // only restore modifiers for click events
        // if let Some(prev) = prev
        //     && prev.key == action.key
        //     && prev.modifiers == action.modifiers
        //     && prev.value == 1
        //     && aciton.value == 0
        // {
        flags = action.modifiers;
        // }

        // prev = Some(action);
        let action = action.to_key_action();
        let _ = next.send_all(InputEvent::Raw(action.to_input_ev()));
    }

    for action in release_restore_modifiers(&flags, _flags) {
        let _ = next.send_all(InputEvent::Raw(action.to_input_ev()));
    }
}

pub async fn handle_callback<'a, State>(
    ev: &EvdevInputEvent,
    handler: Arc<PyObject>,
    args: Option<Vec<PythonArgument>>,
    transformer: Arc<XKBTransformer>,
    modifiers: &KeyModifierFlags,
    next: Vec<Arc<dyn LinkDst>>,
    state: MutexGuard<'a, State>,
) {
    drop(state);
    // release all trigger mods before running the callback
    if !next.is_empty() {
        let new_events = release_restore_modifiers(&modifiers, &KeyModifierFlags::default());
        new_events.iter().cloned().for_each(|ev| next.send_all(InputEvent::Raw(ev.to_input_ev())));
    }
    run_python_handler(handler.clone(), args, ev.clone(), transformer, next.clone()).await;
    // restore all trigger mods after running the callback
    if !next.is_empty() {
        let new_events = release_restore_modifiers(&KeyModifierFlags::default(), &modifiers);
        new_events.iter().cloned().for_each(|ev| next.send_all(InputEvent::Raw(ev.to_input_ev())));
    }
}
