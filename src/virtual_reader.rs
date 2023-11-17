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
use crate::virtual_writer::VirtualWriter;
use crate::writer::Writer;
use crate::xkb::UTFToRawInputTransformer;

pub enum ReaderMessage {
    AddSubscriber(Subscriber),
    SendEvent(EvdevInputEvent),
    SendRawEvent(EvdevInputEvent),
}

// bitflags! {
//     pub struct TransformerFlags: u8 {
//         // do not remap the event, pretend that mappings do not exist
//         const RAW_MODE = 0x01;
//     }
// }


#[pyclass]
pub struct VirtualReader {
    id: String,
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    transformer: UTFToRawInputTransformer,
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

        let transformer = UTFToRawInputTransformer::new(
            options.get("model").and_then(|x| x.extract().ok()),
            options.get("layout").and_then(|x| x.extract().ok()),
            options.get("variant").and_then(|x| x.extract().ok()),
            options.get("options").and_then(|x| x.extract().ok()),
        );

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            subscriber,
            transformer,
        })
    }

    pub fn send(&mut self, val: String) {
        if let Some(subscriber) = self.subscriber.load().deref() {
            let actions = parse_key_sequence_py(val.as_str(), &self.transformer).unwrap().to_key_actions();

            for action in actions {
                subscriber.handle(&self.id, InputEvent::Raw(action.to_input_ev()));
            }
        }
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