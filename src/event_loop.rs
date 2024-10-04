use std::thread;

use pyo3::types::PyTuple;
use pyo3::{IntoPy, Py, PyAny, Python};

use crate::*;

#[derive(Debug)]
pub enum PythonArgument {
    String(String),
    Number(i32),
}

type Args = Vec<PythonArgument>;

pub fn args_to_py(py: Python<'_>, args: Args) -> &PyTuple {
    PyTuple::new(
        py,
        args.into_iter().map(|x| match x {
            PythonArgument::String(x) => x.into_py(py),
            PythonArgument::Number(x) => x.into_py(py),
        }),
    )
}

pub struct EventLoop {
    thread_handle: Option<thread::JoinHandle<()>>,
    callback_tx: tokio::sync::mpsc::Sender<(Py<PyAny>, Option<Args>)>,
}

impl EventLoop {
    pub fn new() -> Self {
        // TODO add exit channel
        let (callback_tx, mut callback_rx) = tokio::sync::mpsc::channel(128);
        let thread_handle = thread::spawn(move || {
            pyo3_asyncio::tokio::get_runtime().block_on(async move {
                // use std::time::Instant;
                // let now = Instant::now();
                Python::with_gil(|py| {
                    pyo3_asyncio::tokio::run::<_, ()>(py, async move {
                        loop {
                            let (callback_object, args): (Py<PyAny>, Option<Args>) =
                                callback_rx.recv().await.expect("python runtime error: event loop channel is closed");

                            Python::with_gil(|py| {
                                let args = args_to_py(py, args.unwrap_or_default());

                                let asyncio = py
                                    .import("asyncio")
                                    .expect("python runtime error: failed to import 'asyncio', is it installed?");

                                let is_async_callback: bool = asyncio
                                    .call_method1("iscoroutinefunction", (callback_object.as_ref(py),))
                                    .expect("python runtime error: 'iscoroutinefunction' lookup failed")
                                    .extract()
                                    .expect("python runtime error: 'iscoroutinefunction' call failed");

                                if is_async_callback {
                                    let coroutine = callback_object
                                        .call(py, args, None)
                                        .expect("python runtime error: failed to call async callback");

                                    let event_loop = pyo3_asyncio::tokio::get_current_loop(py)
                                        .expect("python runtime error: failed to get the event loop");
                                    let coroutine = event_loop
                                        .call_method1("create_task", (coroutine,))
                                        .expect("python runtime error: failed to create task");

                                    // tasks only actually get run if we convert the coroutine to a rust future, even though we don't use it...
                                    if let Err(err) = pyo3_asyncio::tokio::into_future(coroutine) {
                                        eprintln!("an uncaught error was thrown by the python callback: {}", err);
                                        std::process::exit(1);
                                    }
                                } else {
                                    if let Err(err) = callback_object.call(py, args, None) {
                                        eprintln!("an uncaught error was thrown by the python callback: {}", err);
                                        std::process::exit(1);
                                    }
                                }
                            });
                        }
                    })
                    .expect("python runtime error: failed to start the event loop");
                });
                // let elapsed = now.elapsed();
            });
        });

        EventLoop { thread_handle: Some(thread_handle), callback_tx }
    }
    pub fn execute(&self, callback_object: Py<PyAny>, args: Option<Args>) {
        self.callback_tx.try_send((callback_object, args)).expect(&ApplicationError::TooManyEvents.to_string());
    }
}

lazy_static! {
    pub static ref EVENT_LOOP: Mutex<EventLoop> = Mutex::new(EventLoop::new());
}
