use super::suffix_tree::SuffixTree;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use nom::Slice;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

type Mappings = SuffixTree<RuntimeAction>;

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(derive_new::new)]
struct State {
    name: String,
    transformer: Arc<XKBTransformer>,
    #[new(default)]
    prev: HashMap<Uuid, Arc<dyn LinkSrc>>,
    #[new(default)]
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
    #[new(default)]
    mappings: Mappings,
    #[new(default)]
    modifiers: KeyModifierFlags,
    #[new(default)]
    window: Vec<char>,
}

#[pyclass]
pub struct TextMapper {
    pub id: Uuid,
    pub link: Arc<TextMapperLink>,
    ev_tx: tokio::sync::mpsc::Sender<InputEvent>,
    state: Arc<Mutex<State>>,
}

#[pymethods]
impl TextMapper {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(py: Python, kwargs: Option<PyBound<PyDict>>) -> PyResult<Py<Self>> {
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

        let id = Uuid::new_v4();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::channel(64);
        let state = Arc::new(Mutex::new(State::new(name, transformer)));
        let link = Arc::new(TextMapperLink::new(id, ev_tx.clone(), state.clone()));

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

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if from.len() > 32 {
            return Err(PyRuntimeError::new_err("'from' side cannot be longer than 32 character"));
        }

        let from_seq: Vec<KeyClickActionWithMods> = parse_key_sequence(&from, Some(&state.transformer))
            .map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'from' side:\n{}",
                    ApplicationError::KeyParse(err.to_string()),
                ))
            })?
            .into_iter()
            .map(|x| match x {
                ParsedKeyAction::KeyClickAction(x) => Some(x),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| PyRuntimeError::new_err("invalid key sequence"))?;

        let to = if to.bind(py).is_callable() {
            RuntimeAction::PythonCallback(Arc::new(to))
        } else {
            let to = to.extract::<String>(py).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::InvalidInputType { type_: "String".to_string() }
                ))
            })?;
            let to = parse_key_sequence(&to, Some(&state.transformer)).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::KeySequenceParse(err.to_string()),
                ))
            })?;

            RuntimeAction::ActionSequence(to.to_key_actions_with_mods())
        };

        state.mappings.insert(from, to);

        Ok(())
    }

    pub fn snapshot(&self, existing: Option<&TextMapperSnapshot>) -> Option<TextMapperSnapshot> {
        let mut state = self.state.blocking_lock();
        if let Some(existing) = existing {
            state.mappings = existing.mappings.clone();
            return None;
        }
        Some(TextMapperSnapshot { mappings: state.mappings.clone() })
    }

    pub fn link_to(&mut self, target: &PyBound<PyAny>) -> PyResult<()> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
        target.link_from(self.link.clone());
        self.link.link_to(target);
        Ok(())
    }

    pub fn unlink_to(&mut self, py: Python, target: &PyBound<PyAny>) -> PyResult<bool> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
        target.unlink_from(&self.id);
        let ret = self.link.unlink_to(target.id()).map_err(err_to_py)?;
        Ok(ret)
    }

    pub fn unlink_to_all(&mut self) {
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

    pub fn unlink_from_all(&mut self) {
        let mut state = self.state.blocking_lock();
        for l in state.prev.values_mut() {
            l.unlink_to(&self.id);
        }
        state.prev.clear();
    }

    pub fn unlink_all(&mut self) {
        self.unlink_from_all();
        self.unlink_to_all();
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

    pub fn reset(&mut self) {
        self.state.blocking_lock().mappings.clear();
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
}

impl Drop for TextMapper {
    fn drop(&mut self) {
        self.unlink_from_all();
        self.unlink_to_all();
    }
}

#[derive(Clone, derive_new::new)]
pub struct TextMapperLink {
    id: Uuid,
    ev_tx: Sender<InputEvent>,
    state: Arc<Mutex<State>>,
    #[new(default)]
    py_object: OnceLock<Arc<PyObject>>,
}

impl LinkSrc for TextMapperLink {
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

impl LinkDst for TextMapperLink {
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

#[pyclass]
pub struct TextMapperSnapshot {
    mappings: Mappings,
}

async fn handle(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut state = _state.lock().await;
    // let mut state = &mut *_state;
    if !state.next.is_empty() {
        state.next.send_all(raw_ev.clone());
    }
    let ev = match raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    match ev {
        EvdevInputEvent { event_code: EventCode::EV_KEY(KEY_BACKSPACE), value: 1, .. } => {
            state.window.pop();
        }
        EvdevInputEvent { event_code: EventCode::EV_KEY(KEY_DELETE), value: 1, .. } => {
            state.window.remove(0);
        }
        // key event
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
            if ev.value == 0 {
                let key = state.transformer.raw_to_utf(&key, state.modifiers);

                if let Some(key) = key {
                    state.window.push(key.chars().next().unwrap());
                    // TODO set window size dynamically
                    if state.window.len() > 32 {
                        state.window.remove(0);
                    }

                    let mut hit = None;

                    for i in (0..state.window.len()).rev() {
                        let search: String = state.window.iter().skip(i).collect();
                        if let Some(x) = state.mappings.get(&search) {
                            hit = Some((x.clone(), search.len()));
                        }
                    }

                    if let Some((to, from_len)) = hit {
                        state.window.clear();

                        if !state.next.is_empty() {
                            for _ in 0..from_len {
                                state.next.send_all(InputEvent::Raw(Key::from(KEY_BACKSPACE).to_input_ev(1)));
                                state.next.send_all(InputEvent::Raw(Key::from(KEY_BACKSPACE).to_input_ev(0)));
                            }
                        }

                        match to {
                            RuntimeAction::ActionSequence(seq) => {
                                handle_seq2(&seq, &state.modifiers, &state.next, SeqModifierRestoreMode::Default);
                            }
                            RuntimeAction::PythonCallback(handler) => {
                                // delay the callback until the backspace events are processed
                                tokio::time::sleep(Duration::from_millis(10 * from_len as u64)).await;

                                handle_callback(
                                    &ev,
                                    handler.clone(),
                                    None,
                                    state.transformer.clone(),
                                    &state.modifiers.clone(),
                                    state.next.values().cloned().collect(),
                                    state,
                                )
                                .await;
                            }
                            RuntimeAction::NOP => {}
                        }

                        // return after handled match
                        return;
                    }
                }
            }

            // event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));
            state.modifiers.update_from_action(&KeyAction::from_input_ev(&ev));
        }
        _ => {}
    }
}
