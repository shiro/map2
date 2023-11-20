use std::sync::mpsc;

use python::*;

use crate::*;
use crate::device::*;
use crate::device::virt_device::DeviceCapabilities;
use crate::subscriber::{Subscribable, Subscriber};

struct Inner {
    out_ev_tx: mpsc::Sender<EvdevInputEvent>,
}

impl Subscribable for Inner {
    fn handle(&self, id: &str, ev: InputEvent) {
        match ev {
            InputEvent::Raw(ev) => {
                let _ = self.out_ev_tx.send(ev);
            }
        }
    }
}


#[pyclass]
pub struct Writer {
    exit_tx: Option<oneshot::Sender<()>>,
    out_ev_tx: mpsc::Sender<EvdevInputEvent>,
    thread_handle: Option<thread::JoinHandle<Result<()>>>,
    inner: Arc<Inner>,
}

#[pymethods]
impl Writer {
    #[new]
    #[pyo3(signature = (* * kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new()
        };


        let device_name = match options.get("name") {
            Some(option) => option.extract::<String>()
                .map_err(|_| PyRuntimeError::new_err("'name' must be a string"))?,
            None => "Virtual map2 output".to_string()
        };

        let mut capabilities = DeviceCapabilities::new();
        if let Some(_capabilities) = options.get("capabilities") {
            let _capabilities: HashMap<&str, &PyAny> = _capabilities.extract()
                .map_err(|_| PyRuntimeError::new_err("the 'capabilities' object must be a dict"))?;

            if _capabilities.contains_key("keyboard") { capabilities.enable_keyboard(); }
            if _capabilities.contains_key("buttons") { capabilities.enable_buttons(); }
            if _capabilities.contains_key("rel") { capabilities.enable_rel(); }
            if _capabilities.contains_key("abs") { capabilities.enable_abs(); }
        } else {
            capabilities.enable_keyboard();
            capabilities.enable_buttons();
            capabilities.enable_rel();
        }

        let device_init_policy = match options.get("clone_from") {
            Some(_existing_dev_fd) => {
                let existing_dev_fd = _existing_dev_fd.extract::<String>()
                    .map_err(|_| PyRuntimeError::new_err("the 'clone_from' option must be a string"))?;

                if options.get("capabilities").is_some() {
                    return Err(PyRuntimeError::new_err("expected only one of: 'clone_from', 'capabilities'"));
                }

                virtual_output_device::DeviceInitPolicy::CloneExistingDevice(existing_dev_fd)
            }
            None => {
                virtual_output_device::DeviceInitPolicy::NewDevice(device_name, capabilities)
            }
        };

        let (exit_tx, exit_rx) = oneshot::channel();
        let (out_ev_tx, out_ev_rx) = mpsc::channel::<EvdevInputEvent>();

        let thread_handle = thread::spawn(move || {
            // grab udev device
            let mut output_device = virtual_output_device::init_virtual_output_device(&device_init_policy)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

            loop {
                if let Ok(()) = exit_rx.try_recv() { return Ok(()); }

                while let Ok(ev) = out_ev_rx.try_recv() {
                    let mut syn = SYN_REPORT.clone();
                    syn.time.tv_sec = ev.time.tv_sec;
                    syn.time.tv_usec = ev.time.tv_usec;


                    #[cfg(not(feature = "integration"))]
                    {
                        let _ = output_device.send(&ev);
                        let _ = output_device.send(&syn);
                    }
                    #[cfg(feature = "integration")]
                    global::TEST_PIPE.lock().unwrap().push(testing::TestEvent::WriterOutEv(ev.clone()));

                    // this is a hack that stops successive events to not get registered
                    if let EventCode::EV_KEY(_) = ev.event_code {
                        thread::sleep(Duration::from_millis(1));
                    }
                }

                thread::sleep(Duration::from_millis(10));
                thread::yield_now();
            }
        });

        let inner = Arc::new(Inner {
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
}

impl Writer {
    pub fn subscribe(&self) -> Subscriber {
        self.inner.clone()
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if let Some(exit_tx) = self.exit_tx.take() {
            let _ = exit_tx.send(());
            let _ = self.thread_handle.take().unwrap().try_timed_join(Duration::from_millis(5000));
        }
    }
}