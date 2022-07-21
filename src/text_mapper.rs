use std::array::IntoIter;
use std::borrow::Borrow;
use std::collections::vec_deque::VecDeque;
use std::fmt::{Debug, format};
use std::sync::mpsc;
use std::thread;
use evdev_rs::enums::EventType;

use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::*;
use crate::device::virt_device::DeviceCapabilities;
use crate::mapper::Mapper;
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::reader::{Reader, ReaderMessage, Subscriber};


struct Node {
    children: Option<HashMap<String, Node>>,
    seq: Option<String>,
}

#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(String, String),
}

#[pyclass]
pub struct TextMapper {
    id: String,
    pub route: Vec<String>,
    msg_tx: mpsc::Sender<ControlMessage>,
    pub reader_msg_tx: mpsc::Sender<ReaderMessage>,
}

#[pymethods]
impl TextMapper {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(subscribable: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
            None => HashMap::new()
        };

        let id = Uuid::new_v4().to_string();

        let mut route;
        let reader_msg_tx;

        if let Ok(reader) = subscribable.extract::<PyRefMut<Reader>>() {
            route = vec![id.clone()];
            reader_msg_tx = reader.msg_tx.clone();
        } else if let Ok(mapper) = subscribable.extract::<PyRefMut<Mapper>>() {
            route = mapper.route.clone();
            route.push(id.clone());
            reader_msg_tx = mapper.reader_msg_tx.clone();
        } else {
            return Err(PyTypeError::new_err("invalid type for argument subscribable"));
        }

        let (msg_tx, msg_rx) = mpsc::channel();

        let mut handle = Self {
            route,
            id,
            reader_msg_tx,
            msg_tx,
        };

        handle.init_callback(msg_rx);

        Ok(handle)
    }

    pub fn map(&mut self, py: Python, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();
        let mut to = parse_key_action_with_mods_py(&to).unwrap();
        // self._map_internal(from, vec![to])?;

        Ok(())
    }
}


impl TextMapper {
    pub fn subscribe(&mut self, ev_tx: mpsc::Sender<InputEvent>) {
        self.reader_msg_tx.send(ReaderMessage::AddSubscriber(Subscriber {
            route: self.route.clone(),
            ev_tx,
        })).unwrap();
    }

    fn init_callback(&mut self, control_rx: mpsc::Receiver<ControlMessage>) {
        let mut key_window = VecDeque::new();

        let mut lookup = HashMap::new();
        lookup.insert("KEY_A".to_string(),
                      Node {
                          children: Some(HashMap::from([(
                              "KEY_N".to_string(),
                              Node {
                                  children: None,
                                  seq: Some("hello".to_string()),
                              }
                          )])),
                          seq: None,
                      });

        self.reader_msg_tx.send(ReaderMessage::AddTransformer(self.id.clone(), Box::new(move |ev, flags| {
            if ev.value != 1 {
                return vec![ev];
            }

            if key_window.len() >= 20 {
                key_window.pop_back();
            }
            key_window.push_front(ev.event_code.to_string());

            let mut i = 1;
            // let node_ref;

            let foo = ev.event_code.to_string();

            if let Some(mut node_ref) = lookup.get(&foo) {
                loop {
                    if let Some(children) = &node_ref.children {
                        if let Some(n) = children.get(key_window.get(i).unwrap_or(&"_".to_string())) {
                            node_ref = n;
                            i = i + 1;
                        } else {
                            break;
                        }
                    } else {
                        if let Some(seq) = &node_ref.seq {
                            let mut out = vec![];
                            for _ in 0..i - 1 {
                                out.push(KeyAction { key: Key::from_str(&EventType::EV_KEY, "KEY_BACKSPACE").unwrap(), value: TYPE_DOWN }.to_input_ev());
                                out.push(SYN_REPORT.clone());
                                out.push(KeyAction { key: Key::from_str(&EventType::EV_KEY, "KEY_BACKSPACE").unwrap(), value: TYPE_UP }.to_input_ev());
                                out.push(SYN_REPORT.clone());
                            }

                            for char in seq.chars() {
                                let key_name = format!("KEY_{}", char.to_uppercase());
                                out.push(KeyAction { key: Key::from_str(&EventType::EV_KEY, &key_name).unwrap(), value: TYPE_DOWN }.to_input_ev());
                                out.push(SYN_REPORT.clone());
                                out.push(KeyAction { key: Key::from_str(&EventType::EV_KEY, &key_name).unwrap(), value: TYPE_UP }.to_input_ev());
                                out.push(SYN_REPORT.clone());
                            }

                            return out;
                        } else {
                            unreachable!();
                        }
                    }
                }
            }

            vec![ev]
        }))).unwrap();
    }
}

