use std::array::IntoIter;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;

use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::virtual_input_device::Sendable;
use crate::device::virtual_output_device::{init_virtual_output_device, VirtualOutputDevice};
use crate::event_loop::EventLoop;
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::reader::{Reader, ReaderMessage, Subscriber, TransformerFlags, TransformerFn};

lazy_static! {
    static ref EVENT_LOOP: Mutex<EventLoop> = Mutex::new(EventLoop::new());
}


#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(KeyActionWithMods, RuntimeAction),
}


#[pyclass]
pub struct Mapper {
    id: String,
    pub route: Vec<String>,
    msg_tx: mpsc::Sender<ControlMessage>,
    pub reader_msg_tx: mpsc::Sender<ReaderMessage>,
}

#[pymethods]
impl Mapper {
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

        let (control_tx, control_rx) = mpsc::channel();

        let mut handle = Self {
            route,
            id,
            reader_msg_tx,
            msg_tx: control_tx,
        };

        handle.init_callback(control_rx);

        Ok(handle)
    }

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        if let Ok(to) = to.extract::<String>(py) {
            let from = parse_key_action_with_mods_py(&from).unwrap();
            let mut to = parse_key_sequence_py(&to).unwrap();

            self._map_internal(from, to)?;
            return Ok(());
        }

        let is_callable = to.cast_as::<PyAny>(py)
            .map_or(false, |obj| obj.is_callable());

        if is_callable {
            self._map_callback(from, to)?;
            return Ok(());
        }

        Err(PyTypeError::new_err("unknown type"))
    }

    pub fn map_key(&mut self, py: Python, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();
        let mut to = parse_key_action_with_mods_py(&to).unwrap();
        self._map_internal(from, vec![to])?;

        Ok(())
    }
}

impl Mapper {
    fn init_callback(&mut self, control_rx: mpsc::Receiver<ControlMessage>) {
        let mut mappings = Mappings::new();
        let mut state = State::new();

        fn release_restore_modifiers(state: &mut State, from_flags: &KeyModifierFlags, to_flags: &KeyModifierFlags, to_type: &i32) -> Vec<InputEvent> {
            let actual_state = &state.modifiers;
            let mut output_events = vec![];

            // takes into account the actual state of a modifier and decides whether to release/restore it or not
            let mut release_or_restore_modifier = |is_actual_down: &bool, key: &Key| {
                if *to_type == 1 { // restore mods if actual mod is still pressed
                    if *is_actual_down {
                        output_events.push(
                            KeyAction { key: *key, value: *to_type }.to_input_ev()
                        );
                    }
                } else { // release mods if actual mod is still pressed (prob. always true since it was necessary to trigger the mapping)
                    if *is_actual_down {
                        output_events.push(
                            KeyAction { key: *key, value: *to_type }.to_input_ev()
                        );
                    }
                }
            };

            if from_flags.ctrl && !to_flags.ctrl {
                release_or_restore_modifier(&actual_state.left_ctrl, &*KEY_LEFT_CTRL);
                release_or_restore_modifier(&actual_state.right_ctrl, &*KEY_RIGHT_CTRL);
            }
            if from_flags.shift && !to_flags.shift {
                release_or_restore_modifier(&actual_state.left_shift, &*KEY_LEFT_SHIFT);
                release_or_restore_modifier(&actual_state.right_shift, &*KEY_RIGHT_SHIFT);
            }
            if from_flags.alt && !to_flags.alt {
                release_or_restore_modifier(&actual_state.left_alt, &*KEY_LEFT_ALT);
                release_or_restore_modifier(&actual_state.right_alt, &*KEY_RIGHT_ALT);
            }
            if from_flags.meta && !to_flags.meta {
                release_or_restore_modifier(&actual_state.left_meta, &*KEY_LEFT_META);
                release_or_restore_modifier(&actual_state.right_meta, &*KEY_RIGHT_META);
            }

            if output_events.len() > 0 {
                output_events.push(SYN_REPORT.clone());
            }

            output_events

            // TODO eat keys we just released, un-eat keys we just restored
        }

        self.reader_msg_tx.send(ReaderMessage::AddTransformer(self.id.clone(), Box::new(move |ev, flags| {
            while let Ok(msg) = control_rx.try_recv() {
                match msg {
                    ControlMessage::AddMapping(from, to) => {
                        mappings.insert(from, to);
                    }
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

            if !flags.contains(TransformerFlags::RAW_MODE) {
                if let Some(runtime_action) = mappings.get(&from_key_action) {
                    let mut events = vec![];

                    match runtime_action {
                        RuntimeAction::ActionSequence(seq) => {
                            for action in seq {
                                match action {
                                    RuntimeKeyAction::KeyAction(key_action) => {
                                        let ev = key_action.to_input_ev();
                                        events.push(ev);
                                        events.push(SYN_REPORT.clone());
                                    }
                                    RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                        let mut new_events = release_restore_modifiers(&mut state, from_flags, to_flags, to_type);
                                        events.append(&mut new_events);
                                    }
                                }
                            }
                        }
                        RuntimeAction::PythonCallback(from_modifiers, callback_object) => {
                            // always release all trigger mods before running the callback
                            let mut new_events = release_restore_modifiers(&mut state, from_modifiers, &KeyModifierFlags::new(), &TYPE_UP);
                            events.append(&mut new_events);

                            EVENT_LOOP.lock().unwrap().execute(callback_object.clone());
                        }
                        RuntimeAction::NOP => {}
                    }

                    return events;
                }
            }

            event_handlers::update_modifiers(&mut state, &KeyAction::from_input_ev(&ev));
            vec![ev]
        }))).unwrap();
    }

    pub fn subscribe(&mut self, ev_tx: mpsc::Sender<InputEvent>) {
        self.reader_msg_tx.send(ReaderMessage::AddSubscriber(Subscriber {
            route: self.route.clone(),
            ev_tx,
        })).unwrap();
    }

    fn _map_callback(&mut self, from: String, to: PyObject) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();

        match from {
            ParsedKeyAction::KeyAction(from) => {
                let _ = self.msg_tx.send(ControlMessage::AddMapping(from, RuntimeAction::PythonCallback(from.modifiers, to)));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                let _ = self.msg_tx.send(ControlMessage::AddMapping(from.to_key_action(1),
                                                                    RuntimeAction::PythonCallback(from.modifiers, to)));
                let _ = self.msg_tx.send(ControlMessage::AddMapping(from.to_key_action(0),
                                                                    RuntimeAction::NOP));
                let _ = self.msg_tx.send(ControlMessage::AddMapping(from.to_key_action(2),
                                                                    RuntimeAction::NOP));
            }
        }

        Ok(())
    }

    fn _map_internal(&mut self, from: ParsedKeyAction, mut to: Vec<ParsedKeyAction>) -> PyResult<()> {
        match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    // action to click
                    if let ParsedKeyAction::KeyClickAction(to) = to {
                        let mapping = map_action_to_click(&from, &to);
                        let _ = self.msg_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
                        return Ok(());
                    }
                    // action to action
                    if let ParsedKeyAction::KeyAction(to) = to {
                        let mapping = map_action_to_action(&from, &to);
                        let _ = self.msg_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
                        return Ok(());
                    }
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                let _ = self.msg_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    match to.remove(0) {
                        // click to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mappings = map_click_to_click(&from, &to);

                            IntoIter::new(mappings).for_each(|(from, to)| {
                                let _ = self.msg_tx.send(ControlMessage::AddMapping(from, to));
                            });
                            return Ok(());
                        }
                        // click to action
                        ParsedKeyAction::KeyAction(to) => {
                            let mappings = map_click_to_action(&from, &to);
                            IntoIter::new(mappings).for_each(|(from, to)| {
                                let _ = self.msg_tx.send(ControlMessage::AddMapping(from, to));
                            });
                            return Ok(());
                        }
                    };
                }

                // click to seq
                let mappings = map_click_to_seq(from, to);
                IntoIter::new(mappings).for_each(|(from, to)| {
                    let _ = self.msg_tx.send(ControlMessage::AddMapping(from, to));
                });
            }
        }

        Ok(())
    }
}