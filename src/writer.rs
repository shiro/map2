use std::array::IntoIter;
use std::thread;
use crate::device::virtual_output_device::init_virtual_output_device;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::sync::mpsc;
use pyo3::types::PyDict;

use crate::*;
use crate::mapper::Mapper;
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::reader::{Reader};

pub trait EventRoutable {
    fn route(&mut self) -> Result<mpsc::Receiver<InputEvent>>;
}

#[pyclass]
#[derive(Clone)]
pub struct EventRoute {}

impl EventRoutable for EventRoute {
    fn route(&mut self) -> Result<mpsc::Receiver<InputEvent>> { panic!("hey, listen") }
}

#[pyclass]
pub struct Writer {
    exit_tx: Option<oneshot::Sender<()>>,
    thread_handle: Option<std::thread::JoinHandle<Result<()>>>,
    // message_tx: std::sync::mpsc::Sender<ControlMessage>,
    // out_ev_tx: mpsc::Sender<InputEvent>,
}

// #[derive(FromPyObject)]
// enum Subscribable {
//     #[pyo3(transparent)]
//     Reader(Reader),
//     #[pyo3(transparent)]
//     Mapper(Mapper),
// }

#[pymethods]
impl Writer {
    #[new]
    #[args(kwargs = "**")]
    // pub fn new(reader: &mut Reader, kwargs: Option<&PyDict>) -> PyResult<Self> {
    pub fn new(subscribable: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract()
                .map_err(|_| PyTypeError::new_err("the options object must be a dict"))?,
            None => HashMap::new()
        };

        let device_name = match options.get("name") {
            Some(option) => option.extract::<String>()
                .map_err(|_| PyTypeError::new_err("'name' must be a string"))?,
            None => "Virtual map2 output".to_string()
        };

        // let r = input.call_method(py, "route", (), None)
        //     .unwrap()
        // .extract(py)
        // .unwrap();
        // .unwrap();
        // let router = reader.route();
        // router.ev_rx;

        // let ev_rx = &input.call_method0("route")
        //     .map_err(|err| PyTypeError::new_err(err.to_string()))?
        //     .extract::<RoutableObj>().unwrap().inner.unwrap();

        let (exit_tx, exit_rx) = oneshot::channel();
        // let (message_tx, message_rx) = mpsc::channel();
        let (out_ev_tx, out_ev_rx) = mpsc::channel();


        // let reader = match subscribable {
        //     Subscribable::Reader(reader) => reader
        //     Subscribable::Mapper(_) => { unimplemented!() }
        // }
        if let Ok(mut reader) = subscribable.extract::<PyRefMut<Reader>>() {
            reader.subscribe(out_ev_tx);
        } else if let Ok(mut mapper) = subscribable.extract::<PyRefMut<Mapper>>() {
            mapper.subscribe(out_ev_tx);
        } else {
            return Err(PyTypeError::new_err("Invalid type for argument subscribable"));
        }


        let thread_handle = thread::spawn(move || {
            // let mut state = State::new();
            // let mut mappings = Mappings::new();

            // make new dev
            let mut output_device = init_virtual_output_device(&device_name)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

            loop {
                if let Ok(()) = exit_rx.try_recv() { return Ok(()); }

                while let Ok(ev) = out_ev_rx.try_recv() {
                    let _ = output_device.send(&ev);
                }

                // while let Ok(ev) = ev_rx.try_recv() {
                //     event_handlers::handle_stdin_ev(&mut state, ev, &mappings, &mut output_device).unwrap();
                // }

                // while let Ok(msg) = message_rx.try_recv() {
                //     event_handlers::handle_control_message(msg, &mut state, &mut mappings);
                // }

                thread::sleep(time::Duration::from_millis(2));
                thread::yield_now();
            }
        });

        let handle = Self {
            exit_tx: Some(exit_tx),
            thread_handle: Some(thread_handle),
            // message_tx,
            // out_ev_tx,
        };

        Ok(handle)
    }


    // pub fn send(&mut self, val: String) {
    //     let actions = parse_key_sequence_py(val.as_str()).unwrap();
    //
    //     for action in actions.to_key_actions() {
    //         self.out_ev_tx.send(action.to_input_ev()).unwrap();
    //         self.out_ev_tx.send(SYN_REPORT.clone()).unwrap();
    //     }
    // }
    //
    // pub fn send_modifier(&mut self, val: String) -> PyResult<()> {
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
    //     if [*KEY_LEFT_CTRL, *KEY_RIGHT_CTRL, *KEY_LEFT_ALT, *KEY_RIGHT_ALT, *KEY_LEFT_SHIFT, *KEY_RIGHT_SHIFT, *KEY_LEFT_META, *KEY_RIGHT_META]
    //         .contains(&action.key) {
    //         let _ = self.message_tx.send(ControlMessage::UpdateModifiers(*action));
    //     } else {
    //         return Err(PyValueError::new_err("key action needs to be a modifier event"));
    //     }
    //
    //     self.out_ev_tx.send(action.to_input_ev()).unwrap();
    //     self.out_ev_tx.send(SYN_REPORT.clone()).unwrap();
    //     Ok(())
    // }
    //
    // pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
    //     if let Ok(to) = to.extract::<String>(py) {
    //         let from = parse_key_action_with_mods_py(&from).unwrap();
    //         let mut to = parse_key_sequence_py(&to).unwrap();
    //
    //         self._map_internal(from, to)?;
    //         return Ok(());
    //     }
    //
    //     let is_callable = to.cast_as::<PyAny>(py)
    //         .map_or(false, |obj| obj.is_callable());
    //
    //     if is_callable {
    //         self._map_callback(from, to)?;
    //         return Ok(());
    //     }
    //
    //     Err(PyTypeError::new_err("unknown type"))
    // }
    //
    // pub fn map_key(&mut self, py: Python, from: String, to: String) -> PyResult<()> {
    //     let from = parse_key_action_with_mods_py(&from).unwrap();
    //     let mut to = parse_key_action_with_mods_py(&to).unwrap();
    //     self._map_internal(from, vec![to])?;
    //
    //     Ok(())
    // }
}

// impl Writer {
//     fn _map_callback(&mut self, from: String, to: PyObject) -> PyResult<()> {
//         let from = parse_key_action_with_mods_py(&from).unwrap();
//
//         match from {
//             ParsedKeyAction::KeyAction(from) => {
//                 let _ = self.message_tx.send(ControlMessage::AddMapping(from, RuntimeAction::PythonCallback(from.modifiers, to)));
//             }
//             ParsedKeyAction::KeyClickAction(from) => {
//                 let _ = self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(1),
//                                                                         RuntimeAction::PythonCallback(from.modifiers, to)));
//                 let _ = self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(0),
//                                                                         RuntimeAction::NOP));
//                 let _ = self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(2),
//                                                                         RuntimeAction::NOP));
//             }
//         }
//
//         Ok(())
//     }
//
//     fn _map_internal(&mut self, from: ParsedKeyAction, mut to: Vec<ParsedKeyAction>) -> PyResult<()> {
//         match from {
//             ParsedKeyAction::KeyAction(from) => {
//                 if to.len() == 1 {
//                     let to = to.remove(0);
//                     // action to click
//                     if let ParsedKeyAction::KeyClickAction(to) = to {
//                         let mapping = map_action_to_click(&from, &to);
//                         let _ = self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
//                         return Ok(());
//                     }
//                     // action to action
//                     if let ParsedKeyAction::KeyAction(to) = to {
//                         let mapping = map_action_to_action(&from, &to);
//                         let _ = self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
//                         return Ok(());
//                     }
//                 }
//
//                 // action to seq
//                 let mapping = map_action_to_seq(from, to);
//                 let _ = self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
//             }
//             ParsedKeyAction::KeyClickAction(from) => {
//                 if to.len() == 1 {
//                     match to.remove(0) {
//                         // click to click
//                         ParsedKeyAction::KeyClickAction(to) => {
//                             let mappings = map_click_to_click(&from, &to);
//
//                             IntoIter::new(mappings).for_each(|(from, to)| {
//                                 let _ = self.message_tx.send(ControlMessage::AddMapping(from, to));
//                             });
//                             return Ok(());
//                         }
//                         // click to action
//                         ParsedKeyAction::KeyAction(to) => {
//                             let mappings = map_click_to_action(&from, &to);
//                             IntoIter::new(mappings).for_each(|(from, to)| {
//                                 let _ = self.message_tx.send(ControlMessage::AddMapping(from, to));
//                             });
//                             return Ok(());
//                         }
//                     };
//                 }
//
//                 // click to seq
//                 let mappings = map_click_to_seq(from, to);
//                 IntoIter::new(mappings).for_each(|(from, to)| {
//                     let _ = self.message_tx.send(ControlMessage::AddMapping(from, to));
//                 });
//             }
//         }
//
//         Ok(())
//     }
// }

impl Drop for Writer {
    fn drop(&mut self) {
        if let Some(exit_tx) = self.exit_tx.take() {
            exit_tx.send(()).unwrap();
            self.thread_handle.take().unwrap().try_timed_join(Duration::from_millis(5000)).unwrap();
        }
    }
}
