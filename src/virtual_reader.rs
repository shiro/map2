use ::oneshot;
use evdev_rs::enums::EV_REL;
use evdev_rs::TimeVal;
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
use crate::xkb_transformer_registry::XKB_TRANSFORMER_REGISTRY;

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

        let transformer = if kbd_model.is_some()
            || kbd_layout.is_some()
            || kbd_variant.is_some()
            || kbd_options.is_some() {
            Some(
                XKB_TRANSFORMER_REGISTRY.get(kbd_model, kbd_layout, kbd_variant, kbd_options)
                    .map_err(|err| PyRuntimeError::new_err(err.to_string()))?
            )
        } else { None };

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            subscriber,
            transformer,
        })
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        if let Some(subscriber) = self.subscriber.load().deref() {
            let actions = parse_key_sequence_py(val.as_str(), self.transformer.as_deref())
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

    pub fn send_rel(&mut self, axis: &str, delta: i32) -> PyResult<()> {
        let axis = match &*axis.to_uppercase() {
            "X" => { EV_REL::REL_X }
            "Y" => { EV_REL::REL_Y }
            _ => { return Err(PyRuntimeError::new_err("axis must be one of: 'X', 'Y'")); }
        };

        if let Some(subscriber) = self.subscriber.load().deref() {
            subscriber.handle(&self.id, InputEvent::Raw(
                EvdevInputEvent::new(&TimeVal { tv_sec: 0, tv_usec: 0 }, &EventCode::EV_REL(axis), delta)
            ));
        }
        Ok(())
    }

    pub fn link(&mut self, target: &PyAny) -> PyResult<()> { self._link(target) }
}

impl VirtualReader {
    linkable!();
}