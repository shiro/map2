use crate::python::*;
use crate::subscriber::SubscriberNew;
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

pub fn run_python_handler(
    handler: &PyObject,
    args: Option<Vec<PythonArgument>>,
    ev: &EvdevInputEvent,
    transformer: &Arc<XKBTransformer>,
    next: Option<&SubscriberNew>,
) {
    let ret = Python::with_gil(|py| -> Result<()> {
        let asyncio = py.import("asyncio").expect("python runtime error: failed to import 'asyncio', is it installed?");

        let is_async_callback: bool = asyncio
            .call_method1("iscoroutinefunction", (handler.as_ref(py),))
            .expect("python runtime error: 'iscoroutinefunction' lookup failed")
            .extract()
            .expect("python runtime error: 'iscoroutinefunction' call failed");

        if is_async_callback {
            EVENT_LOOP.lock().unwrap().execute(handler.clone(), args);
            Ok(())
        } else {
            let args = args_to_py(py, args.unwrap_or(vec![]));
            let ret = handler.call(py, args, None).map_err(|err| anyhow!("{}", err)).and_then(|ret| {
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
                    let seq = parse_key_sequence(&ret, Some(transformer))?;

                    if let Some(next) = next {
                        for action in seq.to_key_actions() {
                            let _ = next.send(InputEvent::Raw(action.to_input_ev()));
                        }
                    }
                }
                Some(PythonReturn::Bool(ret)) if ret => {
                    if let Some(next) = next {
                        let _ = next.send(InputEvent::Raw(ev.clone()));
                    }
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
}
