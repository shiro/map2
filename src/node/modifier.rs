use self::event_loop::PythonArgument;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeAction;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use futures::executor::block_on;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, MutexGuard};

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(derive_new::new)]
struct State {
    name: String,
    transformer: Arc<XKBTransformer>,
    #[new(default)]
    prev: HashMap<Uuid, Arc<dyn LinkSrc>>,
    #[new(default)]
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
    key: Key,
    #[new(default)]
    click_action: Option<RuntimeAction>,
}

#[pyclass]
pub struct Modifier {
    pub id: Uuid,
    pub link: Arc<MapperLink>,
    ev_tx: tokio::sync::mpsc::Sender<InputEvent>,
    state: Arc<Mutex<State>>,
}

#[pymethods]
impl Modifier {
    #[new]
    #[pyo3(signature = (key, **kwargs))]
    pub fn new(py: Python, key: String, kwargs: Option<PyBound<PyDict>>) -> PyResult<Py<Self>> {
        let options: HashMap<String, Bound<PyAny>> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new(),
        };

        let name = options
            .get("name")
            .and_then(|x| x.extract().ok())
            .unwrap_or(format!("text mapper {}", node_util::get_id_and_incremen(&ID_COUNTER)))
            .to_string();
        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(err_to_py)?;

        parse_key2(&key, |raw| None);

        let key = parse_key(&key, Some(&transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!("failed to parse key '{}'", ApplicationError::KeyParse(err.to_string()),))
        })?;

        let id = Uuid::new_v4();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::channel(64);
        let state = Arc::new(Mutex::new(State::new(name, transformer, key)));
        let link = Arc::new(MapperLink::new(id, ev_tx.clone(), state.clone()));

        {
            let state = state.clone();
            get_runtime().spawn(async move {
                loop {
                    let ev = ev_rx.recv().await;
                    match ev {
                        Some(ev) => handle(state.clone(), ev).await,
                        None => return,
                    }
                }
            });
        }

        let _link = link.clone();
        let _self = Py::new(py, Self { id, link, ev_tx, state })?;
        _link.py_object.set(Arc::new(_self.to_object(py)));
        Ok(_self)
    }

    pub fn link_to(&self, target: &PyBound<PyAny>) -> PyResult<()> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
        target.link_from(self.link.clone());
        self.link.link_to(target);
        Ok(())
    }

    pub fn unlink_to(&self, py: Python, target: &PyBound<PyAny>) -> PyResult<bool> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
        target.unlink_from(&self.id);
        let ret = self.link.unlink_to(target.id()).map_err(err_to_py)?;
        Ok(ret)
    }

    pub fn unlink_to_all(&self) {
        let mut state = self.state.blocking_lock();
        for l in state.next.values_mut() {
            l.unlink_from(&self.id);
        }
        state.next.clear();
    }

    pub fn link_from(&mut self, target: &PyBound<PyAny>) -> PyResult<()> {
        let target = node_to_link_src(target).ok_or_else(|| PyRuntimeError::new_err("expected a source node"))?;
        target.link_to(self.link.clone());
        self.link.link_from(target);
        Ok(())
    }

    pub fn unlink_from(&mut self, target: &PyBound<PyAny>) -> PyResult<bool> {
        let target = node_to_link_src(target).ok_or_else(|| PyRuntimeError::new_err("expected a source node"))?;
        target.unlink_to(&self.id);
        let ret = self.link.unlink_from(target.id()).map_err(err_to_py)?;
        Ok(ret)
    }

    pub fn unlink_from_all(&self) {
        let mut state = self.state.blocking_lock();
        for l in state.prev.values_mut() {
            l.unlink_to(&self.id);
        }
        state.prev.clear();
    }

    pub fn unlink_all(&self) {
        self.unlink_from_all();
        self.unlink_to_all();
    }

    pub fn insert_after(&self, target: &PyBound<PyAny>) -> PyResult<()> {
        let mut state = self.state.blocking_lock();

        let target_src =
            node_to_link_src(target).ok_or_else(|| PyRuntimeError::new_err("expected a source+destination node"))?;
        let target_dst =
            node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a source+destination node"))?;

        for (_, node) in state.next.drain() {
            target_src.link_to(node);
        }
        drop(state);
        self.link_to(target);

        Ok(())
    }

    pub fn name(&self) -> String {
        self.state.blocking_lock().name.clone()
    }

    pub fn next(&self, py: Python) -> Vec<PyObject> {
        self.state.blocking_lock().next.values().map(|v| v.py_object().to_object(py)).collect()
    }

    pub fn prev(&self, py: Python) -> Vec<PyObject> {
        self.state.blocking_lock().prev.values().map(|v| v.py_object().to_object(py)).collect()
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        let actions = parse_key_sequence(val.as_str(), Some(&state.transformer))
            .map_err(|err| ApplicationError::KeySequenceParse(err.to_string()).into_py())?
            .to_key_actions();
        for action in actions {
            self.ev_tx
                .try_send(InputEvent::Raw(action.to_input_ev()))
                .expect(&ApplicationError::TooManyEvents.to_string());
        }
        Ok(())
    }

    pub fn send_after(&mut self, val: String) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        let actions = parse_key_sequence(val.as_str(), Some(&state.transformer))
            .map_err(|err| ApplicationError::KeySequenceParse(err.to_string()).into_py())?
            .to_key_actions();
        for action in actions {
            state.next.send_all(InputEvent::Raw(action.to_input_ev()));
        }
        Ok(())
    }
}

impl Modifier {
    //
}

impl Drop for Modifier {
    fn drop(&mut self) {
        self.unlink_from_all();
        self.unlink_to_all();
    }
}

#[derive(Clone, derive_new::new)]
pub struct MapperLink {
    id: Uuid,
    ev_tx: Sender<InputEvent>,
    state: Arc<Mutex<State>>,
    #[new(default)]
    py_object: OnceLock<Arc<PyObject>>,
}

impl LinkSrc for MapperLink {
    fn id(&self) -> &Uuid {
        &self.id
    }
    fn link_to(&self, node: Arc<dyn LinkDst>) -> Result<()> {
        self.state.blocking_lock().next.insert(*node.id(), node);
        Ok(())
    }
    fn unlink_to(&self, id: &Uuid) -> Result<bool> {
        Ok(self.state.blocking_lock().next.remove(id).is_some())
    }
    fn py_object(&self) -> Arc<PyObject> {
        self.py_object.get().unwrap().clone()
    }
}

impl LinkDst for MapperLink {
    fn id(&self) -> &Uuid {
        &self.id
    }
    fn link_from(&self, node: Arc<dyn LinkSrc>) -> Result<()> {
        self.state.blocking_lock().prev.insert(*node.id(), node);
        Ok(())
    }
    fn unlink_from(&self, id: &Uuid) -> Result<bool> {
        Ok(self.state.blocking_lock().prev.remove(id).is_some())
    }
    fn send(&self, ev: InputEvent) -> Result<()> {
        self.ev_tx.try_send(ev).map_err(|err| ApplicationError::TooManyEvents.into_py())?;
        Ok(())
    }
    fn py_object(&self) -> Arc<PyObject> {
        self.py_object.get().unwrap().clone()
    }
}

async fn handle(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut state = _state.lock().await;
    let ev = match &raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    match ev {
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. }
            if EventCode::EV_KEY(*key) == state.key.event_code =>
        {
            println!("hi {:?}", ());
            // let from_key_action = KeyActionWithMods {
            //     key: Key { event_code: ev.event_code },
            //     value: ev.value,
            //     modifiers: state.modifiers,
            // };

            // if let Some(runtime_action) = state.mappings.get(&from_key_action) {
            //     match runtime_action {
            //         RuntimeAction::ActionSequence(seq) => {
            //             let mode = get_mode(&state.mappings, &from_key_action, seq);
            //             handle_seq2(seq, &state.modifiers, &state.next, mode);
            //         }
            //         RuntimeAction::PythonCallback(handler) => {
            //             handle_callback(
            //                 &ev,
            //                 handler.clone(),
            //                 Some(python_callback_args(
            //                     &EventCode::EV_KEY(*key),
            //                     &state.modifiers,
            //                     *value,
            //                     &state.transformer,
            //                 )),
            //                 state.transformer.clone(),
            //                 &state.modifiers.clone(),
            //                 state.next.values().cloned().collect(),
            //                 state,
            //             )
            //             .await;
            //         }
            //         RuntimeAction::NOP => {}
            //     }
            //
            //     return;
            // }

            // let args =
            //     Some(python_callback_args(&EventCode::EV_KEY(*key), &state.modifiers, *value, &state.transformer));

            // state.modifiers.update_from_action(&KeyAction::from_input_ev(&ev));

            // if let Some(handler) = state.fallback_handler.as_ref() {
            //     let handler = handler.clone();
            //     let transformer = state.transformer.clone();
            //     let next = state.next.values().cloned().collect();
            //     drop(state);
            //     run_python_handler(handler, args, ev.clone(), transformer, next).await;
            //     return;
            // }
        }
        _ => {}
    }

    state.next.send_all(raw_ev);
}
