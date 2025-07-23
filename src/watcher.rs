use crate::conversions::get_py_type;
use crate::device::virtual_input_device::MatcherValue;
use crate::python::*;
use crate::python_util::*;
use crate::subscriber::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use ::oneshot;
use device::virtual_input_device::{DeviceMatcher, NativeDeviceInfo};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(derive_new::new)]
struct State {
    name: String,
    #[new(default)]
    devices: HashSet<NativeDeviceInfo>,
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

        let filters = if let Some(v) = options.get("filters") {
            let filter_vec = if let Ok(v) = v.extract::<Vec<PyObject>>() {
                v
            } else if let Ok(_) = v.extract::<PyObject>() {
                vec![v.clone().unbind()]
            } else {
                return Err(ApplicationError::InvalidNamedInputType {
                    name: "filters".to_string(),
                    actual_type: get_py_type(v),
                    expected_type: "list[str] | list[DeviceMatcher]".to_string(),
                }
                .into_py())?;
            };

            filter_vec
                .into_iter()
                .map(|v| {
                    if let Ok(v) = v.extract::<String>(py) {
                        Ok(DeviceMatcher { path: Some(MatcherValue::Str(v)), properties: Default::default() })
                    } else if let Ok(matcher) = v.extract::<DeviceMatcher>(py) {
                        Ok(matcher)
                    } else {
                        Err(ApplicationError::InvalidNamedInputType {
                            name: "filters".to_string(),
                            actual_type: get_py_type(v.bind(py)),
                            expected_type: "list[str] | list[DeviceMatcher]".to_string(),
                        }
                        .into_py())
                    }
                })
                .collect::<PyResult<Vec<DeviceMatcher>>>()?
        } else {
            vec![]
        };

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
        {
            use device::virtual_input_device::NativeDeviceEvent;
            use device::virtual_input_device::watch_udev_inputs;

            let state = state.clone();
            let handler = Arc::new(move |ev: NativeDeviceEvent| {
                let mut state = state.lock().unwrap();
                match ev {
                    NativeDeviceEvent::AddDevice(info) => {
                        if let Some(handler) = state.on_connect_handler.as_ref() {
                            Python::with_gil(|py| {
                                let info = device_info_to_py(py, &info);
                                handler.call(py, (info,), None)
                            });
                        }
                        state.devices.insert(info);
                    }
                    NativeDeviceEvent::RemoveDevice(info) => {
                        if let Some(handler) = state.on_disconnect_handler.as_ref() {
                            Python::with_gil(|py| {
                                let info = device_info_to_py(py, &info);
                                handler.call(py, (info,), None)
                            });
                        }
                        state.devices.remove(&info);
                    }
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

    #[getter]
    pub fn devices(&self, py: Python) -> Vec<PyObject> {
        let state = self.state.lock().unwrap();
        state.devices.iter().map(|info| device_info_to_py(py, info)).collect()
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
