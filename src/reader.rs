use core::mem;
use std::rc::Rc;
use std::sync::{mpsc, MutexGuard};
use std::thread;

use ::oneshot;
use pyo3::exceptions::PyTypeError;
use pyo3::number::sub;
use pyo3::prelude::*;
use pyo3::types::{PyDict};
use writer::EventRoute;

use crate::*;
use crate::device::virtual_input_device::Sendable;
use crate::writer::EventRoutable;


pub struct Subscriber {
    pub ev_tx: mpsc::Sender<InputEvent>,
    pub route: Vec<String>,
}

pub type TransformerFn = Box<dyn FnMut(InputEvent, &mut State) -> Vec<InputEvent> + Send>;

pub enum ReaderMessage {
    AddSubscriber(Subscriber),
    AddTransformer(String, TransformerFn),
}

#[pyclass]
pub struct Reader {
    reader_exit_tx: Option<oneshot::Sender<()>>,
    reader_thread_handle: Option<thread::JoinHandle<Result<()>>>,

    mapper_exit_tx: Option<oneshot::Sender<()>>,
    mapper_thread_handle: Option<thread::JoinHandle<Result<()>>>,

    pub msg_tx: mpsc::Sender<ReaderMessage>,
}


impl<T> Sendable<T> for async_channel::Sender<T> {
    fn send(&self, t: T) {
        let _ = self.send(t);
    }
}

impl<T> Sendable<T> for mpsc::Sender<T> {
    fn send(&self, t: T) {
        let _ = self.send(t);
    }
}

#[pymethods]
impl Reader {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = kwargs
            .ok_or_else(|| PyTypeError::new_err("no options provided"))?
            .extract()
            .map_err(|_| PyTypeError::new_err("the options object must be a dict"))?;

        let patterns: Vec<&str> = options.get("patterns")
            .ok_or_else(|| PyTypeError::new_err("'patterns' is required but was not provided"))?
            .extract()
            .map_err(|_| PyTypeError::new_err("'patterns' must be a list"))?;

        let (reader_exit_tx, reader_exit_rx) = oneshot::channel();
        let (mapper_exit_tx, mapper_exit_rx) = oneshot::channel();
        let (ev_tx, ev_rx) = mpsc::channel();
        let (msg_tx, msg_rx) = mpsc::channel();

        let reader_thread_handle = grab_udev_inputs(&patterns, ev_tx, reader_exit_rx)
            .map_err(|err| PyTypeError::new_err(err.to_string()))?;

        let mapper_thread_handle = thread::spawn(move || {
            let mut state = State::new();
            let mut subscribers = vec![];
            let mut transformers = HashMap::new();

            loop {
                if let Ok(()) = mapper_exit_rx.try_recv() { return Ok(()); }

                while let Ok(msg) = msg_rx.try_recv() {
                    match msg {
                        ReaderMessage::AddSubscriber(subscriber) => { subscribers.push(subscriber); }
                        ReaderMessage::AddTransformer(id, transformer_fn) => { transformers.insert(id, transformer_fn); }
                    }
                }

                while let Ok(ev) = ev_rx.try_recv() {
                    for s in subscribers.iter_mut() {
                        let mut events = vec![ev.clone()];

                        for id in s.route.iter() {
                            let transformer: Option<&mut TransformerFn> = transformers.get_mut(id);
                            match transformer {
                                Some(transformer) => {
                                    let mut next_events = vec![];
                                    for ev in events.into_iter() {
                                        let mut new_events = transformer.deref_mut().call_mut((ev, &mut state));
                                        next_events.append(&mut new_events);
                                    }
                                    events = next_events;
                                }
                                None => { panic!("transformer not found") }
                            }
                        }

                        for ev in events.into_iter() {
                            // println!("ev: {:?}", ev);
                            let _ = s.ev_tx.send(ev);
                        }
                    }
                }
            }
        });

        let handle = Self {
            reader_exit_tx: Some(reader_exit_tx),
            reader_thread_handle: Some(reader_thread_handle),
            mapper_exit_tx: Some(mapper_exit_tx),
            mapper_thread_handle: Some(mapper_thread_handle),
            // ev_rx: Some(ev_rx),
            // router: Rc::new(EvRouter { ev_rx: Some(ev_rx) }),
            msg_tx,
        };

        Ok(handle)
    }
}

impl Reader {
    pub fn subscribe(&mut self, ev_tx: mpsc::Sender<InputEvent>) {
        self.msg_tx.send(ReaderMessage::AddSubscriber(Subscriber {
            route: vec![],
            ev_tx,
        })).unwrap();
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.reader_exit_tx.take().unwrap().send(());
        let _ = self.mapper_exit_tx.take().unwrap().send(());

        let _ = self.reader_thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
        let _ = self.mapper_thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
    }
}
