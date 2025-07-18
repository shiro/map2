use ::oneshot;
use device::virtual_input_device::{DeviceMatcher, NativeDeviceInfo};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::event::InputEvent;
use crate::python::*;
use crate::subscriber::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(derive_new::new)]
struct State {
    name: String,
    #[new(default)]
    devices: Arc<Mutex<HashSet<NativeDeviceInfo>>>,
    #[new(default)]
    on_connect_handler: Option<Arc<PyObject>>,
    #[new(default)]
    on_disconnect_handler: Option<Arc<PyObject>>,
}

#[pyclass]
pub struct Watcher {
    pub id: Uuid,
    state: Arc<Mutex<State>>,
    #[cfg(not(feature = "integration"))]
    reader_exit_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[pymethods]
impl Watcher {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(py: Python, kwargs: Option<PyBound<PyDict>>) -> PyResult<Self> {
        let options: HashMap<String, Bound<PyAny>> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new(),
        };

        let mut filters = vec![];

        if let Some(v) = options.get("filters") {
            if let Ok(v) = v.extract::<Vec<PyObject>>() {
                for v in v.into_iter() {
                    let filter = if let Ok(value) = v.extract::<String>(py) {
                        DeviceMatcher::new().tap_mut(|v| {
                            v.insert("path".to_string(), value);
                        })
                    } else if let Ok(matcher) = v.extract::<HashMap<String, String>>(py) {
                        matcher
                    } else {
                        return Err(PyRuntimeError::new_err("'filters' must be of type 'string[]?'"));
                    };
                    filters.push(filter);
                }
            } else {
                return Err(PyRuntimeError::new_err("'patterns' must be of type 'string[]?'"));
            }
        }

        let name = options
            .get("name")
            .and_then(|x| x.extract().ok())
            .unwrap_or(format!("reader {}", node_util::get_id_and_incremen(&ID_COUNTER)))
            .to_string();

        #[cfg(not(feature = "integration"))]
        let (reader_exit_tx, reader_exit_rx) = tokio::sync::oneshot::channel();

        let id = Uuid::new_v4();
        let state = Arc::new(Mutex::new(State::new(name)));

        #[cfg(not(feature = "integration"))]
        let reader_thread_handle = {
            use device::virtual_input_device::watch_udev_inputs;
            use device::virtual_input_device::NativeDeviceEvent;

            let state = state.clone();
            let handler = Arc::new(move |ev: NativeDeviceEvent| {
                let mut state = state.lock().unwrap();
                match ev {
                    NativeDeviceEvent::AddDevice(info) => {
                        if let Some(handler) = state.on_connect_handler.as_ref() {
                            Python::with_gil(|py| handler.call(py, (info.to_hash_map(),), None));
                        }
                        state.devices.lock().unwrap().insert(info);
                    }
                    NativeDeviceEvent::RemoveDevice(info) => {
                        if let Some(handler) = state.on_disconnect_handler.as_ref() {
                            Python::with_gil(|py| handler.call(py, (info.to_hash_map(),), None));
                        }
                        state.devices.lock().unwrap().remove(&info);
                    }
                    NativeDeviceEvent::InputEvent(_) => unreachable!(),
                };
            });

            watch_udev_inputs(filters, handler, reader_exit_rx).map_err(err_to_py)?
        };

        Ok(Self {
            id,
            state,
            #[cfg(not(feature = "integration"))]
            reader_exit_tx: Some(reader_exit_tx),
        })
    }

    pub fn on_connect(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.lock().unwrap();
        if handler.bind(py).is_none() {
            state.on_connect_handler = None;
            return Ok(());
        }
        if !handler.bind(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }

        // call the handler with all known devices
        for info in state.devices.lock().unwrap().iter() {
            Python::with_gil(|py| handler.call(py, (info.to_hash_map(),), None));
        }

        state.on_connect_handler = Some(Arc::new(handler));
        Ok(())
    }

    pub fn on_disconnect(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.lock().unwrap();
        if handler.bind(py).is_none() {
            state.on_disconnect_handler = None;
            return Ok(());
        }
        if !handler.bind(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.on_disconnect_handler = Some(Arc::new(handler));
        Ok(())
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        #[cfg(not(feature = "integration"))]
        let _ = self.reader_exit_tx.take().map(|v| {
            v.send(());
        });
    }
}
