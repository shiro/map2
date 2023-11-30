use std::fmt::Debug;
use std::sync::mpsc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;

struct Node {
    children: Option<HashMap<u8, Node>>,
    seq: Option<Vec<KeyAction>>,
}

#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(Vec<u8>, Vec<KeyAction>),
}

#[pyclass]
pub struct TextMapper {
    id: String,
    msg_tx: mpsc::Sender<ControlMessage>,
    // pub route: Vec<String>,
    // pub reader_msg_tx: mpsc::Sender<ReaderMessage>,
}

#[pymethods]
impl TextMapper {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
            None => HashMap::new()
        };

        let id = Uuid::new_v4().to_string();

        // let mut route;
        // let reader_msg_tx;

        // if let Ok(reader) = subscribable.extract::<PyRefMut<Reader>>() {
        //     route = vec![id.clone()];
        //     reader_msg_tx = reader.msg_tx.clone();
        // } else if let Ok(mapper) = subscribable.extract::<PyRefMut<Mapper>>() {
        //     route = mapper.route.clone();
        //     route.push(id.clone());
        //     reader_msg_tx = mapper.reader_msg_tx.clone();
        // } else {
        //     return Err(PyTypeError::new_err("invalid type for argument subscribable"));
        // }

        let (msg_tx, msg_rx) = mpsc::channel();

        let handle = Self {
            id,
            // route,
            // reader_msg_tx,
            msg_tx,
        };

        // handle.init_callback(msg_rx);

        Ok(handle)
    }

    // pub fn map(&mut self, from: String, to: String) -> PyResult<()> {
    //     let from = parse_key_sequence_py(&from, &self.tr).unwrap();
    //     let to = parse_key_sequence_py(&to).unwrap();
    //
    //     let from = from
    //         .to_key_actions()
    //         .into_iter()
    //         .filter(|action| {
    //             action.value == 0
    //         })
    //         .map(|action| match action.key.event_code {
    //             EventCode::EV_KEY(key) => { key as u8 }
    //             _ => panic!("only keys are supported")
    //         })
    //         .collect();
    //
    //     self._map_internal(from, to)?;
    //     Ok(())
    // }
}


impl TextMapper {
//     pub fn subscribe(&mut self, ev_tx: mpsc::Sender<InputEvent>) {
//         self.reader_msg_tx.send(ReaderMessage::AddSubscriber(Subscriber {
//             route: self.route.clone(),
//             ev_tx,
//         })).unwrap();
//     }

    // fn init_callback(&mut self, control_rx: mpsc::Receiver<ControlMessage>) {
    //     let mut key_window = VecDeque::new();
    //     let mut lookup = HashMap::new();
    //     let mut key_window_len = 0;
    //     let backspace = Key::from_str(&EventType::EV_KEY, "KEY_BACKSPACE").unwrap();
    //
    //     self.reader_msg_tx.send(ReaderMessage::AddTransformer(self.id.clone(), Box::new(move |ev, flags| {
    //         if ev.value != 1 {
    //             return vec![ev];
    //         }
    //
    //         if ev.event_code == backspace.event_code {
    //             key_window.pop_front();
    //             return vec![ev];
    //         }
    //
    //         if let Ok(msg) = control_rx.try_recv() {
    //             match msg {
    //                 ControlMessage::AddMapping(from, to) => {
    //                     if from.len() > key_window_len {
    //                         key_window_len = from.len();
    //                     }
    //
    //                     let mut inner = Node {
    //                         children: None,
    //                         seq: Some(to),
    //                     };
    //
    //                     let from_len = from.len();
    //                     for (i, code) in from.into_iter().enumerate() {
    //                         // ignore last
    //                         if i == from_len - 1 {
    //                             lookup.insert(code, inner);
    //                             break;
    //                         }
    //
    //                         inner = Node {
    //                             children: Some(HashMap::from([(code, inner)])),
    //                             seq: None,
    //                         };
    //                     }
    //                 }
    //             }
    //         }
    //
    //         if key_window_len == 0 {
    //             return vec![ev];
    //         }
    //
    //         if key_window.len() >= key_window_len {
    //             key_window.pop_back();
    //         }
    //
    //         let code = match ev.event_code {
    //             EventCode::EV_KEY(key) => { key as u8 }
    //             _ => panic!("only keys are supported")
    //         };
    //
    //         key_window.push_front(code);
    //         let mut i = 1;
    //
    //         if let Some(mut node_ref) = lookup.get(&code) {
    //             loop {
    //                 if let Some(children) = &node_ref.children {
    //                     let code = match key_window.get(i) {
    //                         Some(code) => code,
    //                         None => break
    //                     };
    //                     if let Some(n) = children.get(code) {
    //                         node_ref = n;
    //                         i = i + 1;
    //                     } else {
    //                         break;
    //                     }
    //                 } else {
    //                     if let Some(seq) = &node_ref.seq {
    //                         let mut out = vec![];
    //                         for _ in 0..i - 1 {
    //                             out.push(KeyAction { key: backspace.clone(), value: TYPE_DOWN }.to_input_ev());
    //                             out.push(SYN_REPORT.clone());
    //                             out.push(KeyAction { key: backspace.clone(), value: TYPE_UP }.to_input_ev());
    //                             out.push(SYN_REPORT.clone());
    //                         }
    //
    //                         for action in seq {
    //                             out.push(action.to_input_ev());
    //                             out.push(SYN_REPORT.clone());
    //                         }
    //
    //                         return out;
    //                     } else {
    //                         unreachable!();
    //                     }
    //                 }
    //             }
    //         }
    //
    //         vec![ev]
    //     }))).unwrap();
    // }

    fn _map_internal(&mut self, from: Vec<u8>, to: Vec<ParsedKeyAction>) -> PyResult<()> {
        self.msg_tx.send(ControlMessage::AddMapping(from, to.to_key_actions()))
            .unwrap();
        Ok(())
    }
}

