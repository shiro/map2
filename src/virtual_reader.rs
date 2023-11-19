use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::virtual_input_device::Sendable;
use crate::event::InputEvent;
use crate::parsing::key_action::ParsedKeyActionVecExt;
use crate::parsing::python::parse_key_sequence_py;
use crate::subscriber::Subscriber;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};

pub enum ReaderMessage {
    AddSubscriber(Subscriber),
    SendEvent(EvdevInputEvent),
    SendRawEvent(EvdevInputEvent),
}


#[pyclass]
pub struct VirtualReader {
    id: String,
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    transformer: Option<Arc<XKBTransformer>>,
    transformer_params: TransformerParams,
}

#[pymethods]
impl VirtualReader {
    #[new]
    #[pyo3(signature = (* * kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
            None => HashMap::new()
        };

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer_params = TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options);

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            subscriber,
            transformer: None,
            transformer_params,
        })
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        self.init_transformer().map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        if let Some(subscriber) = self.subscriber.load().deref() {
            let actions = parse_key_sequence_py(val.as_str(), Some(&self.transformer.as_ref().unwrap()))
                .map_err(|err| PyRuntimeError::new_err(
                    format!("key sequence parse error: {}", err.to_string())
                ))?
                .to_key_actions();

            for action in actions {
                subscriber.handle(&self.id, InputEvent::Raw(action.to_input_ev()));
            }
        }
        Ok(())
    }

    pub fn link(&mut self, target: &PyAny) -> PyResult<()> { self._link(target) }
}

impl VirtualReader {
    linkable!();

    fn init_transformer(&mut self) -> Result<()> {
        if self.transformer.is_none() {
            let transformer = XKB_TRANSFORMER_REGISTRY.get(&self.transformer_params)?;
            self.transformer = Some(transformer);
        }
        Ok(())
    }
}