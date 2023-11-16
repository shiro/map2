use ::oneshot;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::device::virtual_input_device::Sendable;
use crate::event::InputEvent;
use crate::mapper::Mapper;
use crate::parsing::key_action::ParsedKeyActionVecExt;
use crate::parsing::python::parse_key_sequence_py;
use crate::subscriber::Subscriber;
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
    #[args(kwargs = "**")]
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

    pub fn link(&mut self, target: &PyAny) {
        if let Ok(mut target) = target.extract::<PyRefMut<Writer>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Writer(target.inner.clone())))
            );
        } else if let Ok(mut target) = target.extract::<PyRefMut<Mapper>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Mapper(target.inner.clone())))
            );
        }
    }

    pub fn send(&mut self, val: String) {
        if let Some(subscriber) = self.subscriber.load().deref() {
            let actions = parse_key_sequence_py(val.as_str(), &self.transformer).unwrap().to_key_actions();

            for action in actions {
                subscriber.handle(&self.id, InputEvent::Raw(action.to_input_ev()));
            }
        }
    }

    // pub fn send_raw(&mut self, val: String) -> PyResult<()> {
    //     let actions = parse_key_sequence_py(val.as_str())
    //         .unwrap()
    //         .to_key_actions();
    //
    //     if actions.len() != 1 {
    //         return Err(PyValueError::new_err(format!("expected a single key action, got {}", actions.len())));
    //     }
    //
    //     let action = actions.get(0).unwrap();
    //
    //     if ![*KEY_LEFT_CTRL, *KEY_RIGHT_CTRL, *KEY_LEFT_ALT, *KEY_RIGHT_ALT, *KEY_LEFT_SHIFT, *KEY_RIGHT_SHIFT, *KEY_LEFT_META, *KEY_RIGHT_META]
    //         .contains(&action.key) {
    //         return Err(PyValueError::new_err("key action needs to be a modifier event"));
    //     }
    //
    //     // self.msg_tx.send(ReaderMessage::SendRawEvent(action.to_input_ev())).unwrap();
    //
    //     Ok(())
    // }
}