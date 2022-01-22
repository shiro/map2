use std::thread;

use pyo3::{Py, PyAny, Python};
use pyo3::types::PyBool;

pub struct EventLoop {
    thread_handle: Option<std::thread::JoinHandle<()>>,
    callback_tx: tokio::sync::mpsc::Sender<Py<PyAny>>,
}

impl EventLoop {
    pub fn new() -> Self {
        // TODO add exit channel
        let (callback_tx, mut callback_rx) = tokio::sync::mpsc::channel::<Py<PyAny>>(128);
        let thread_handle = thread::spawn(move || {
            pyo3_asyncio::tokio::get_runtime().block_on(async move {
                // use std::time::Instant;
                // let now = Instant::now();
                Python::with_gil(|py| {
                    pyo3_asyncio::tokio::run::<_, ()>(py, async move {
                        loop {
                            let callback_object = callback_rx.recv().await.unwrap();
                            let f = Python::with_gil(|py| {
                                let callback_object = callback_object.cast_as::<PyAny>(py).unwrap();

                                let asyncio = py.import("asyncio").unwrap();
                                let is_async_callback = asyncio
                                    .call_method1("iscoroutinefunction", (callback_object, ))
                                    .unwrap()
                                    .cast_as::<PyBool>()
                                    .unwrap();

                                if is_async_callback.is_true() {
                                    let coroutine = callback_object.call((), None).unwrap();

                                    let event_loop = pyo3_asyncio::tokio::get_current_loop(py).unwrap();
                                    let coroutine = event_loop.call_method1("create_task", (coroutine, )).unwrap();

                                    // tasks only actually get run if we convert the coroutine to a rust future, even though we don't use it...
                                    let _ = pyo3_asyncio::tokio::into_future(coroutine);
                                } else {
                                    let _ = callback_object.call((), None);
                                }
                            });
                        }
                    }).unwrap();
                });
                // let elapsed = now.elapsed();
            });
        });

        EventLoop {
            thread_handle: Some(thread_handle),
            callback_tx,
        }
    }
    pub fn execute(&mut self, callback_object: Py<PyAny>) {
        futures::executor::block_on(self.callback_tx.send(callback_object)).unwrap();
    }
}