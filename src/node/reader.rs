use super::mapper_util::parse_device_filters;
use crate::conversions::extract_with_error;
use crate::conversions::get_py_type;
use crate::device::virtual_input_device::MatcherValue;
use crate::device::virtual_input_device::NativeDeviceInfo;
use crate::device::virtual_input_device::grab_device;
use crate::device::virtual_input_device::watch_udev_inputs;
use crate::event::InputEvent;
use crate::python::*;
use crate::python_util::*;
use crate::subscriber::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use ::oneshot;
use device::virtual_input_device::DeviceMatcher;
use pyo3::IntoPyObjectExt;
use std::hash::{Hash, Hasher};

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(derive_new::new)]
struct State {
    name: String,
    #[new(default)]
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
    #[new(default)]
    devices: HashSet<NativeDeviceInfo>,
    #[new(default)]
    on_connect_handler: Option<Arc<PyObject>>,
    #[new(default)]
    on_disconnect_handler: Option<Arc<PyObject>>,
}

/// Reads input events from sources such as device nodes
#[pyclass]
pub struct Reader {
    pub id: Uuid,
    pub link: Arc<ReaderLink>,
    state: Arc<Mutex<State>>,
    transformer: Arc<XKBTransformer>,
    #[cfg(not(feature = "integration"))]
    reader_exit_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[pymethods]
impl Reader {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(py: Python, kwargs: Option<PyBound<PyDict>>) -> PyResult<Self> {
        let options: HashMap<String, Bound<PyAny>> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new(),
        };

        let filters = if let Some(v) = options.get("filters") { parse_device_filters(py, v)? } else { vec![] };

        let name = extract_with_error::<String>(&options, "name")?
            .unwrap_or_else(|| format!("Reader {}", node_util::get_id_and_incremen(&ID_COUNTER)));

        let kbd_model = extract_with_error::<String>(&options, "model")?;
        let kbd_layout = extract_with_error::<String>(&options, "layout")?;
        let kbd_variant = extract_with_error::<String>(&options, "variant")?;
        let kbd_options = extract_with_error::<String>(&options, "options")?;

        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        #[cfg(not(feature = "integration"))]
        let (reader_exit_tx, reader_exit_rx) = tokio::sync::oneshot::channel();

        let id = Uuid::new_v4();
        let state = Arc::new(Mutex::new(State::new(name)));
        let link = Arc::new(ReaderLink::new(id, state.clone()));

        #[cfg(not(feature = "integration"))]
        if !filters.is_empty() {
            use device::virtual_input_device::NativeDeviceEvent;

            let _state = state.clone();
            let handler = Arc::new(move |ev| {
                let mut state = _state.lock().unwrap();
                let mut device_map = HashMap::new();

                match ev {
                    NativeDeviceEvent::AddDevice(info) => {
                        let _state = _state.clone();
                        let res = grab_device(
                            &info.fd_path,
                            Arc::new(move |ev| {
                                let mut state = _state.lock().unwrap();
                                state.next.send_all(InputEvent::Raw(ev));
                            }),
                        );
                        let abort_handle = match res {
                            Ok(v) => v,
                            Err(err) => {
                                eprintln!("{}", err);
                                std::process::exit(1);
                            }
                        };
                        device_map.insert(info.sys_path.clone(), abort_handle);

                        if let Some(handler) = state.on_connect_handler.as_ref() {
                            Python::with_gil(|py| {
                                let info = device_info_to_py(py, &info);
                                handler.call(py, (info,), None);
                            });
                        }
                        state.devices.insert(info);
                    }
                    NativeDeviceEvent::RemoveDevice(info) => {
                        if let Some(handler) = state.on_disconnect_handler.as_ref() {
                            Python::with_gil(|py| {
                                let info = device_info_to_py(py, &info);
                                handler.call(py, (info,), None);
                            });
                        }
                        state.devices.remove(&info);
                    }
                };
            });

            Some(watch_udev_inputs(filters, handler, reader_exit_rx).map_err(err_to_py)?)
        } else {
            None
        };

        Ok(Self {
            id,
            state,
            transformer,
            link,
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

    pub fn link_to(&self, target: &PyBound<PyAny>) -> PyResult<()> {
        (self.link.clone() as Arc<dyn LinkSrc>).py_link_to(target)
    }

    pub fn unlink_to(&mut self, py: Python, target: &PyBound<PyAny>) -> PyResult<bool> {
        (self.link.clone() as Arc<dyn LinkSrc>).py_unlink_to(target)
    }

    pub fn unlink_to_all(&mut self) {
        (self.link.clone() as Arc<dyn LinkSrc>).py_unlink_to_all();
    }

    pub fn insert_after(&self, target: &PyBound<PyAny>) -> PyResult<()> {
        (self.link.clone() as Arc<dyn LinkSrc>).py_insert_after(target)
    }

    pub fn unlink_all(&mut self) {
        self.unlink_to_all();
    }

    pub fn name(&self) -> String {
        self.state.lock().unwrap().name.clone()
    }

    pub fn next(&self, py: Python) -> Vec<PyObject> {
        self.link.next().into_iter().map(|v| v.py_object().clone_ref(py).into_any()).collect()
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence(val.as_str(), Some(&self.transformer))
            .map_err(|err| ApplicationError::KeySequenceParse(err.to_string()).into_py())?
            .to_key_actions();

        let state = self.state.lock().unwrap();
        for action in actions {
            state.next.send_all(InputEvent::Raw(action.to_input_ev()));
        }
        Ok(())
    }

    #[getter]
    pub fn devices(&self, py: Python) -> Vec<PyObject> {
        let state = self.state.lock().unwrap();
        state.devices.iter().map(|info| device_info_to_py(py, info)).collect()
    }

    #[cfg(feature = "integration")]
    pub fn __test__write_ev(&mut self, ev: String) -> PyResult<()> {
        let ev: EvdevInputEvent = serde_json::from_str(&ev).unwrap();
        let _ = self.state.lock().unwrap().next.send_all(InputEvent::Raw(ev));
        Ok(())
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        #[cfg(not(feature = "integration"))]
        let _ = self.reader_exit_tx.take().map(|v| {
            v.send(());
        });
        self.unlink_all();
    }
}

#[derive(Clone, derive_new::new)]
pub struct ReaderLink {
    id: Uuid,
    state: Arc<Mutex<State>>,
    #[new(default)]
    py_object: OnceLock<Arc<PyObject>>,
}

impl LinkSrc for ReaderLink {
    fn id(&self) -> &Uuid {
        &self.id
    }
    fn link_to(&self, node: Arc<dyn LinkDst>) -> Result<()> {
        self.state.lock().unwrap().next.insert(*node.id(), node);
        Ok(())
    }
    fn unlink_to(&self, id: &Uuid) -> Result<bool> {
        Ok(self.state.lock().unwrap().next.remove(id).is_some())
    }
    fn py_object(&self) -> Arc<PyObject> {
        self.py_object.get().unwrap().clone()
    }
    fn clear_next(&self) -> Vec<Arc<dyn LinkDst>> {
        let mut state = self.state.lock().unwrap();
        state.next.drain().map(|(_, v)| v).collect()
    }
    fn next(&self) -> Vec<Arc<dyn LinkDst>> {
        let state = self.state.lock().unwrap();
        state.next.values().cloned().collect()
    }
}
