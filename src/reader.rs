use ::oneshot;
use device::virtual_input_device::DeviceMatcher;
use std::hash::{Hash, Hasher};

use crate::event::InputEvent;
use crate::python::*;
use crate::subscriber::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;

#[derive(Default)]
struct State {
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
}

#[pyclass]
pub struct Reader {
    pub id: Uuid,
    pub link: Arc<ReaderLink>,
    state: Arc<Mutex<State>>,
    transformer: Arc<XKBTransformer>,
    #[cfg(not(feature = "integration"))]
    reader_exit_tx: Option<oneshot::Sender<()>>,
    #[cfg(not(feature = "integration"))]
    reader_thread_handle: Option<thread::JoinHandle<Result<()>>>,
}

#[pymethods]
impl Reader {
    #[new]
    #[pyo3(signature = (* * kwargs))]
    pub fn new(py: Python, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(options) => options.extract()?,
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

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        #[cfg(not(feature = "integration"))]
        let (reader_exit_tx, reader_exit_rx) = oneshot::channel();

        let id = Uuid::new_v4();
        let state = Arc::new(Mutex::new(State::default()));
        let link = Arc::new(ReaderLink { id, state: state.clone() });

        #[cfg(not(feature = "integration"))]
        let reader_thread_handle = if !filters.is_empty() {
            let state = state.clone();
            let handler = Arc::new(move |_: &str, ev: EvdevInputEvent| {
                // TODO handle error if channel full
                state.lock().unwrap().next.send_all(InputEvent::Raw(ev));
            });

            Some(grab_udev_inputs(filters, handler, reader_exit_rx).map_err(err_to_py)?)
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
            #[cfg(not(feature = "integration"))]
            reader_thread_handle,
        })
    }

    pub fn link_to(&mut self, target: &PyAny) -> PyResult<()> {
        let mut target = node_to_link_dst(target).unwrap();
        target.link_from(self.link.clone());
        self.link.link_to(target);
        Ok(())
    }

    pub fn unlink_to(&mut self, py: Python, target: &PyAny) -> PyResult<bool> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
        target.unlink_from(&self.id);
        let ret = self.link.unlink_to(target.id()).map_err(err_to_py)?;
        Ok(ret)
    }

    pub fn unlink_to_all(&mut self) {
        let mut state = self.state.lock().unwrap();
        for l in state.next.values_mut() {
            l.unlink_from(&self.id);
        }
        state.next.clear();
    }

    pub fn unlink_all(&mut self) {
        self.unlink_to_all();
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

    #[cfg(feature = "integration")]
    pub fn __test__write_ev(&mut self, ev: String) -> PyResult<()> {
        let ev: EvdevInputEvent = serde_json::from_str(&ev).unwrap();
        let _ = self.state.lock().unwrap().next.send_all(InputEvent::Raw(ev));
        Ok(())
    }
}

#[derive(Clone)]
pub struct ReaderLink {
    id: Uuid,
    state: Arc<Mutex<State>>,
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
}
