use std::sync::mpsc;
use std::thread;

use ::oneshot;
use bitflags::bitflags;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::virtual_input_device::Sendable;
use crate::event::InputEvent;
use crate::mapper::{Mapper, MapperInner};
use crate::writer::WriterInner;

pub enum Subscriber {
    Mapper(Arc<MapperInner>),
    Writer(Arc<WriterInner>)
}

impl Subscriber {
    pub(crate) fn handle(&self, id: &str, ev: InputEvent) {
        match self {
            Subscriber::Mapper(target) => { target.handle(id, ev) }
            Subscriber::Writer(target) => { target.handle(id, ev) }
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
    subscriber: Arc<ArcSwapOption<Subscriber>>,
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

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));
        let s = subscriber.clone();

        let handler = Arc::new(move |id: &str, ev: EvdevInputEvent| {
            if let Some(s) = s.load().deref() {
                s.handle(id, InputEvent::Raw(ev));
            }
        });

        let reader_thread_handle = grab_udev_inputs(&patterns, handler, reader_exit_rx)
            .map_err(|err| PyTypeError::new_err(err.to_string()))?;


        let handle = Self {
            reader_exit_tx: Some(reader_exit_tx),
            reader_thread_handle: Some(reader_thread_handle),
            subscriber,
        };

        Ok(handle)
    }

    pub fn link(&mut self, target: &PyAny) {
        if let Ok(mut target) = target.extract::<PyRefMut<Mapper>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Mapper(target.inner.clone())))
            );
        }
    }

    // pub fn send(&mut self, val: String) {
    //     let actions = parse_key_sequence_py(val.as_str()).unwrap().to_key_actions();
    //
    //     for action in actions {
    //         // self.msg_tx.send(ReaderMessage::SendEvent(action.to_input_ev())).unwrap();
    //         // self.msg_tx.send(ReaderMessage::SendEvent(SYN_REPORT.clone())).unwrap();
    //     }
    // }

    // pub fn send_raw(&mut self, val: String) -> PyResult<()> {
    //     let actions = parse_key_sequence_py(val.as_str())
    //         .unwrap()
    //         .to_key_actions();
    //
    //     if actions.len() != 1 {
    //         return Err(PyValueError::new_err(format!("expected a single key action, got {}", actions.len())));
    //     }
    //
    //     let action = actions.get(0).unwrap();
    //
    //     if ![*KEY_LEFT_CTRL, *KEY_RIGHT_CTRL, *KEY_LEFT_ALT, *KEY_RIGHT_ALT, *KEY_LEFT_SHIFT, *KEY_RIGHT_SHIFT, *KEY_LEFT_META, *KEY_RIGHT_META]
    //         .contains(&action.key) {
    //         return Err(PyValueError::new_err("key action needs to be a modifier event"));
    //     }
    //
    //     // self.msg_tx.send(ReaderMessage::SendRawEvent(action.to_input_ev())).unwrap();
    //
    //     Ok(())
    // }
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
        // let _ = self.reader_exit_tx.take().unwrap().send(());
        // let _ = self.mapper_exit_tx.take().unwrap().send(());
        //
        // let _ = self.reader_thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
        // let _ = self.mapper_thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
    }
}
