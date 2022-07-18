use std::array::IntoIter;
use std::sync::mpsc;
use std::thread;

use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::device::virt_device::DeviceCapabilities;
use crate::*;
use crate::device::virtual_output_device::init_virtual_output_device;
use crate::mapper::Mapper;
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::reader::Reader;

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
    out_ev_tx: mpsc::Sender<InputEvent>,
}

#[pymethods]
impl Writer {
    #[new]
    #[args(kwargs = "**")]
    // pub fn new(reader: &mut Reader, kwargs: Option<&PyDict>) -> PyResult<Self> {
    pub fn new(subscribable: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict
                .extract::<HashMap<&str, &PyAny>>()
                .unwrap()
                .remove("options")
                .ok_or_else(|| PyTypeError::new_err("the options object must be a dict"))?
                .extract()
                .map_err(|_| PyTypeError::new_err("the options object must be a dict"))?,
            None => HashMap::new()
        };

        let device_name = match options.get("name") {
            Some(option) => option.extract::<String>()
                .map_err(|_| PyTypeError::new_err("'name' must be a string"))?,
            None => "Virtual map2 output".to_string()
        };

        let mut capabilities = DeviceCapabilities::new();
        if let Some(capabilities_input) = options.get("capabilities") {
            let capabilities_options: HashMap<&str, &PyAny> = capabilities_input.extract()
                .map_err(|_| PyTypeError::new_err("the 'capabilities' object must be a dict"))?;

            if capabilities_options.contains_key("keyboard") { capabilities.enable_keyboard(); }
            if capabilities_options.contains_key("buttons") { capabilities.enable_buttons(); }
            if capabilities_options.contains_key("rel") { capabilities.enable_rel(); }
            if capabilities_options.contains_key("abs") { capabilities.enable_abs(); }
        } else {
            capabilities.enable_keyboard();
            capabilities.enable_buttons();
            capabilities.enable_rel();
        }

        let (exit_tx, exit_rx) = oneshot::channel();
        let (out_ev_tx, out_ev_rx) = mpsc::channel();


        if let Ok(mut reader) = subscribable.extract::<PyRefMut<Reader>>() {
            reader.subscribe(out_ev_tx.clone());
        } else if let Ok(mut mapper) = subscribable.extract::<PyRefMut<Mapper>>() {
            mapper.subscribe(out_ev_tx.clone());
        } else {
            return Err(PyTypeError::new_err("Invalid type for argument subscribable"));
        }


        let thread_handle = thread::spawn(move || {
            // make new dev
            let mut output_device = init_virtual_output_device(&device_name, &capabilities)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

            loop {
                if let Ok(()) = exit_rx.try_recv() { return Ok(()); }

                while let Ok(ev) = out_ev_rx.try_recv() {
                    let _ = output_device.send(&ev);
                }

                thread::sleep(time::Duration::from_millis(2));
                thread::yield_now();
            }
        });

        let handle = Self {
            exit_tx: Some(exit_tx),
            thread_handle: Some(thread_handle),
            out_ev_tx,
        };

        Ok(handle)
    }

    pub fn send(&mut self, val: String) {
        let actions = parse_key_sequence_py(val.as_str()).unwrap();

        for action in actions.to_key_actions() {
            self.out_ev_tx.send(action.to_input_ev()).unwrap();
            self.out_ev_tx.send(SYN_REPORT.clone()).unwrap();
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if let Some(exit_tx) = self.exit_tx.take() {
            exit_tx.send(()).unwrap();
            self.thread_handle.take().unwrap().try_timed_join(Duration::from_millis(5000)).unwrap();
        }
    }
}
