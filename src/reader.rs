use std::sync::mpsc;
use std::thread;

use ::oneshot;
use bitflags::bitflags;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::virtual_input_device::Sendable;
use crate::event::InputEvent;
use crate::mapper::{Mapper, MapperInner};
use crate::parsing::key_action::ParsedKeyActionVecExt;
use crate::parsing::python::parse_key_sequence_py;

// pub struct Subscriber {
//     pub ev_tx: mpsc::Sender<InputEvent>,
//     pub route: Vec<String>,
// }
pub enum Subscriber {
    Mapper(Arc<MapperInner>),
}

impl Subscriber {
    fn handle(&self, ev: InputEvent) {
        match self {
            Subscriber::Mapper(target) => {target.handle(ev)}
        }
    }
}

bitflags! {
    pub struct TransformerFlags: u8 {
        // do not remap the event, pretend that mappings do not exist
        const RAW_MODE = 0x01;
    }
}
// pub type TransformerFn = Box<dyn FnMut(InputEvent, &TransformerFlags) -> Vec<EvdevInputEvent> + Send>;

pub enum ReaderMessage {
    AddSubscriber(Subscriber),
    SendEvent(EvdevInputEvent),
    SendRawEvent(EvdevInputEvent),
}

#[pyclass]
pub struct Reader {
    reader_exit_tx: Option<oneshot::Sender<()>>,
    reader_thread_handle: Option<thread::JoinHandle<Result<()>>>,

    mapper_exit_tx: Option<oneshot::Sender<()>>,
    mapper_thread_handle: Option<thread::JoinHandle<Result<()>>>,

    pub msg_tx: mpsc::Sender<ReaderMessage>,

    // new
    // targets: Vec<Arc<MapperInner>>,
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
        let (evdev_reader_tx, evdev_reader_rx) = mpsc::channel();
        let (msg_tx, msg_rx) = mpsc::channel();

        let reader_thread_handle = grab_udev_inputs(&patterns, evdev_reader_tx, reader_exit_rx)
            .map_err(|err| PyTypeError::new_err(err.to_string()))?;

        let mapper_thread_handle = thread::spawn(move || {
            let mut subscribers = vec![];
            // let mut transformers = HashMap::new();

            fn process_event(ev: EvdevInputEvent, mut subscribers: &mut Vec<Subscriber>) {
                // let mut transformation_cache: HashMap<&String, Vec<InputEvent>> = HashMap::new();

                for s in subscribers.iter_mut() {
                    s.handle(InputEvent::Raw(ev.clone()));
                    //     // non-key events are currently not supported
                    //     match ev.event_code {
                    //         EventCode::EV_KEY(_) => {}
                    //         _ => {
                    //             let _ = s.ev_tx.send(ev.clone());
                    //             continue;
                    //         }
                    //     }
                    //
                    //     let mut events = vec![ev.clone()];
                    //
                    //     for id in s.route.iter() {
                    //         let transformer: Option<&mut TransformerFn> = transformers.get_mut(id);
                    //         match transformer {
                    //             Some(transformer) => {
                    //                 // every transformer is run at most once, cached results are always used
                    //                 if let Some(cached_events) = transformation_cache.get(id) {
                    //                     events = cached_events.clone();
                    //                     continue;
                    //                 }
                    //
                    //                 let mut next_events = vec![];
                    //                 for ev in events.into_iter() {
                    //                     let mut new_events = transformer.deref_mut().call_mut((ev, flags));
                    //                     next_events.append(&mut new_events);
                    //                 }
                    //                 events = next_events;
                    //                 transformation_cache.insert(id, events.clone());
                    //             }
                    //             None => { panic!("transformer not found") }
                    //         }
                    //     }
                    //
                    //     for ev in events.into_iter() {
                    //         let _ = s.ev_tx.send(ev);
                    //     }
                }
            }

            loop {
                if let Ok(()) = mapper_exit_rx.try_recv() { return Ok(()); }

                while let Ok(msg) = msg_rx.try_recv() {
                    match msg {
                        ReaderMessage::AddSubscriber(subscriber) => { subscribers.push(subscriber); }
                        ReaderMessage::SendEvent(ev) => {
                            process_event(ev, &mut subscribers);
                        }
                        ReaderMessage::SendRawEvent(ev) => {
                            process_event(ev, &mut subscribers);
                        }
                    }
                }

                while let Ok(ev) = evdev_reader_rx.try_recv() {
                    process_event(ev, &mut subscribers);
                }
            }
        });

        let handle = Self {
            reader_exit_tx: Some(reader_exit_tx),
            reader_thread_handle: Some(reader_thread_handle),
            mapper_exit_tx: Some(mapper_exit_tx),
            mapper_thread_handle: Some(mapper_thread_handle),
            msg_tx,
        };

        Ok(handle)
    }

    pub fn link(&mut self, target: &PyAny) {
        if let Ok(target) = target.extract::<PyRefMut<Mapper>>() {
            self.msg_tx.send(ReaderMessage::AddSubscriber(
                Subscriber::Mapper(target.inner.clone()))
            ).unwrap();
        }
    }

    pub fn send(&mut self, val: String) {
        let actions = parse_key_sequence_py(val.as_str()).unwrap().to_key_actions();

        for action in actions {
            self.msg_tx.send(ReaderMessage::SendEvent(action.to_input_ev())).unwrap();
            self.msg_tx.send(ReaderMessage::SendEvent(SYN_REPORT.clone())).unwrap();
        }
    }

    pub fn send_raw(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence_py(val.as_str())
            .unwrap()
            .to_key_actions();

        if actions.len() != 1 {
            return Err(PyValueError::new_err(format!("expected a single key action, got {}", actions.len())));
        }

        let action = actions.get(0).unwrap();

        if ![*KEY_LEFT_CTRL, *KEY_RIGHT_CTRL, *KEY_LEFT_ALT, *KEY_RIGHT_ALT, *KEY_LEFT_SHIFT, *KEY_RIGHT_SHIFT, *KEY_LEFT_META, *KEY_RIGHT_META]
            .contains(&action.key) {
            return Err(PyValueError::new_err("key action needs to be a modifier event"));
        }

        self.msg_tx.send(ReaderMessage::SendRawEvent(action.to_input_ev())).unwrap();

        Ok(())
    }
}


impl Reader {
    // pub fn subscribe(&mut self, ev_tx: mpsc::Sender<InputEvent>) {
    //     self.msg_tx.send(ReaderMessage::AddSubscriber(Subscriber {
    //         route: vec![],
    //         ev_tx,
    //     })).unwrap();
    // }
}

impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.reader_exit_tx.take().unwrap().send(());
        let _ = self.mapper_exit_tx.take().unwrap().send(());

        let _ = self.reader_thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
        let _ = self.mapper_thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
    }
}
