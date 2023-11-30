use std::hash::{Hash, Hasher};
use ::oneshot;

use crate::*;
use crate::python::*;
use crate::event::InputEvent;
use crate::subscriber::{Subscriber, add_event_subscription};
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};


#[pyclass]
pub struct Reader {
    pub id: Arc<Uuid>,
    subscriber: Arc<ArcSwapOption<Subscriber>>,
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
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(options) => options.extract()?,
            None => HashMap::new()
        };

        let patterns: Vec<&str> = match options.get("patterns") {
            Some(patterns) => patterns.extract()
                .map_err(|_| PyRuntimeError::new_err("'patterns' must be of type 'string[]?'"))?,
            None => { vec![] }
        };

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        #[cfg(not(feature = "integration"))]
            let (reader_exit_tx, reader_exit_rx) = oneshot::channel();

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));
        let _subscriber = subscriber.clone();

        let id = Arc::new(Uuid::new_v4());

        #[cfg(not(feature = "integration"))]
            let reader_thread_handle = if !patterns.is_empty() {
            let mut h = DefaultHasher::new();
            vec![id.clone()].hash(&mut h);
            let path_hash = h.finish();

            let handler = Arc::new(move |_: &str, ev: EvdevInputEvent| {
                if let Some(subscriber) = _subscriber.load().deref() {
                    let _ = subscriber.send((path_hash, InputEvent::Raw(ev)));
                }
            });

            Some(grab_udev_inputs(&patterns, handler, reader_exit_rx).map_err(err_to_py)?)
        } else { None };

        Ok(Self {
            id,
            subscriber,
            transformer,
            #[cfg(not(feature = "integration"))]
            reader_exit_tx: Some(reader_exit_tx),
            #[cfg(not(feature = "integration"))]
            reader_thread_handle,
        })
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence(val.as_str(), Some(&self.transformer))
            .map_err(|err| PyRuntimeError::new_err(
                format!("key sequence parse error: {}", err.to_string())
            ))?
            .to_key_actions();

        let mut h = DefaultHasher::new();
        vec![self.id.clone()].hash(&mut h);
        let path_hash = h.finish();

        if let Some(subscriber) = self.subscriber.load().deref() {
            for action in actions {
                let _ = subscriber.send((path_hash, InputEvent::Raw(action.to_input_ev())));
            }
        }
        Ok(())
    }

    #[cfg(feature = "integration")]
    pub fn __test__write_ev(&mut self, ev: String) -> PyResult<()> {
        let ev: EvdevInputEvent = serde_json::from_str(&ev).unwrap();

        let mut h = DefaultHasher::new();
        vec![self.id.clone()].hash(&mut h);
        let path_hash = h.finish();

        if let Some(subscriber) = self.subscriber.load().deref() {
            let _ = subscriber.send((path_hash, InputEvent::Raw(ev)));
        };
        Ok(())
    }
}

impl Reader {
    pub fn _link(&mut self, target: &PyAny) -> PyResult<()> {
        use crate::subscriber::*;

        if target.is_none() {
            self.subscriber.store(None);
            return Ok(());
        }

        let target = match add_event_subscription(target) {
            Some(target) => target,
            None => { return Err(PyRuntimeError::new_err("unsupported link target")); }
        };
        self.subscriber.store(Some(Arc::new(target)));
        Ok(())
    }
}