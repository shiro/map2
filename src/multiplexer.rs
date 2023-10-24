use std::thread;
use std::sync::mpsc;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use crate::*;
use crate::reader::Reader;
use crate::writer::{EventRoutable, EventRoute};

enum ControlMessage {
    Subscribe(mpsc::Sender<InputEvent>),
}

#[pyclass]
pub struct Multiplexer {
    exit_tx: Option<oneshot::Sender<()>>,
    thread_handle: Option<std::thread::JoinHandle<Result<()>>>,
    message_tx: std::sync::mpsc::Sender<ControlMessage>,
}

#[pymethods]
impl Multiplexer {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(input: &mut Reader) -> PyResult<Self> {
        let ev_rx = input.route()
            .inner
            .map_err(|err| PyTypeError::new_err(err.to_string()))?;

        let (exit_tx, exit_rx) = oneshot::channel();
        let (message_tx, message_rx) = std::sync::mpsc::channel();

        let thread_handle = thread::spawn(move || {
            let mut subscribers = vec![];
            loop {
                if let Ok(()) = exit_rx.try_recv() { return Ok(()); }

                while let Ok(msg) = message_rx.try_recv() {
                    match msg {
                        ControlMessage::Subscribe(tx) => {
                            subscribers.push(tx);
                        }
                    }
                }

                while let Ok(ev) = ev_rx.try_recv() {
                    for subscriber in &subscribers {
                        subscriber.send(ev.clone());
                    }
                }

                thread::sleep(time::Duration::from_millis(2));
                thread::yield_now();
            }
        });

        Ok(Self {
            exit_tx: Some(exit_tx),
            thread_handle: Some(thread_handle),
            message_tx,
        })
    }
}

impl Multiplexer {
    pub fn route(&mut self) -> Result<mpsc::Receiver<InputEvent>> {
        let (tx, rx) = mpsc::channel();
        self.message_tx.send(ControlMessage::Subscribe(tx));
        Ok(rx)
    }
}


impl Drop for Multiplexer {
    fn drop(&mut self) {
        let _ = self.exit_tx.take().unwrap().send(());
        let _ = self.thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
    }
}
