use std::sync::mpsc;
use std::thread;

use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::*;
use crate::device::virt_device::DeviceCapabilities;

pub struct WriterInner {
    out_ev_tx: mpsc::Sender<EvdevInputEvent>,
}

impl WriterInner {
    pub fn handle(&self, id: &str, ev: InputEvent) {
        match ev {
            InputEvent::Raw(ev) => {
                self.out_ev_tx.send(ev).unwrap();
            }
        }
    }
}


#[pyclass]
pub struct Writer {
    exit_tx: Option<oneshot::Sender<()>>,
    out_ev_tx: mpsc::Sender<EvdevInputEvent>,
    thread_handle: Option<thread::JoinHandle<Result<()>>>,
    pub inner: Arc<WriterInner>,
}

#[pymethods]
impl Writer {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
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

        let device_init_policy = match options.get("clone_from") {
            Some(_existing_dev_fd) => {
                let existing_dev_fd = _existing_dev_fd.extract::<String>()
                    .map_err(|_| PyRuntimeError::new_err("the 'clone_from' option must be a string"))?;

                virtual_output_device::DeviceInitPolicy::CloneExistingDevice(existing_dev_fd)
            }
            None => {
                virtual_output_device::DeviceInitPolicy::NewDevice(device_name, capabilities)
            }
        };

        let (exit_tx, exit_rx) = oneshot::channel();
        let (out_ev_tx, out_ev_rx) = mpsc::channel();

        let thread_handle = thread::spawn(move || {
            // grab udev device
            let mut output_device = virtual_output_device::init_virtual_output_device(&device_init_policy)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

            loop {
                if let Ok(()) = exit_rx.try_recv() { return Ok(()); }

                while let Ok(ev) = out_ev_rx.try_recv() {
                    let _ = output_device.send(&ev);
                    let _ = output_device.send(&SYN_REPORT.clone());

                    // this is a hack that stops successive events to not get registered
                    // maybe if we add proper times to the events it'll be fine...
                    thread::sleep(Duration::from_millis(5));
                }

                thread::sleep(Duration::from_millis(10));
                thread::yield_now();
            }
        });

        let inner = Arc::new(WriterInner {
            out_ev_tx: out_ev_tx.clone(),
        });

        let handle = Self {
            exit_tx: Some(exit_tx),
            out_ev_tx,
            thread_handle: Some(thread_handle),
            inner,
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
}

impl Drop for Writer {
    fn drop(&mut self) {
        if let Some(exit_tx) = self.exit_tx.take() {
            exit_tx.send(()).unwrap();
            self.thread_handle.take().unwrap().try_timed_join(Duration::from_millis(5000)).unwrap();
        }
    }
}