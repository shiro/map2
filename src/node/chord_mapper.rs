use super::mapping_functions::*;
use super::*;
use crate::conversions::{extract_with_error, get_py_type};
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

type Mappings = HashMap<Vec<Key>, RuntimeAction>;

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
    chorded_keys: HashSet<Key>,
    #[new(default)]
    modifiers: KeyModifierFlags,
    // all keys on the stack are included in mappings and are currently pressed
    #[new(default)]
    stack: Vec<Key>,
    #[new(default)]
    ignored_keys: HashSet<Key>,
    #[new(default)]
    pressed_keys: HashSet<Key>,
    #[new(default)]
    interval: Option<tokio::task::JoinHandle<()>>,
}

#[pyclass]
pub struct ChordMapper {
    pub id: Uuid,
    pub link: Arc<ChordMapperLink>,
    ev_tx: tokio::sync::mpsc::Sender<InputEvent>,
    state: Arc<Mutex<State>>,
}

#[pymethods]
impl ChordMapper {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(py: Python, kwargs: Option<PyBound<PyDict>>) -> PyResult<Py<Self>> {
        let kwargs: HashMap<String, Bound<PyAny>> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new(),
        };

        let name = extract_with_error::<String>(&kwargs, "name")?
            .unwrap_or_else(|| format!("ChordMapper {}", node_util::get_id_and_incremen(&ID_COUNTER)));

        let kbd_model = extract_with_error::<String>(&kwargs, "model")?;
        let kbd_layout = extract_with_error::<String>(&kwargs, "layout")?;
        let kbd_variant = extract_with_error::<String>(&kwargs, "variant")?;
        let kbd_options = extract_with_error::<String>(&kwargs, "options")?;

        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(err_to_py)?;

        let id = Uuid::new_v4();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::channel(64);
        let state = Arc::new(Mutex::new(State::new(name, transformer)));
        let link = Arc::new(ChordMapperLink::new(id, ev_tx.clone(), state.clone()));

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
        _link.py_object.set(Arc::new(_self.clone_ref(py).into_any()));
        Ok(_self)
    }

    pub fn map(&mut self, py: Python, from: Vec<String>, to: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        // if from.len() > 32 {
        //     return Err(PyRuntimeError::new_err(
        //         "'from' side cannot be longer than 32 character",
        //     ));
        // }

        let mut from_parsed =
            from.into_iter().map(|x| parse_key(&x, Some(&state.transformer))).collect::<Result<Vec<_>>>().map_err(
                |err| {
                    PyRuntimeError::new_err(format!(
                        "mapping error on the 'from' side:\n{}",
                        ApplicationError::KeyParse(err.to_string()),
                    ))
                },
            )?;

        let to = if to.bind(py).is_callable() {
            RuntimeAction::PythonCallback(Arc::new(to))
        } else {
            let to = to.extract::<String>(py).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::InvalidInputType {
                        name: "to".to_string(),
                        expected_type: "str".to_string(),
                        actual_type: get_py_type(to.bind(py))
                    }
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

        let mut from_parsed = from_parsed.clone();

        // mark chorded keys
        state.chorded_keys.extend(from_parsed.iter().cloned());

        // insert all combinations
        state.mappings.insert(from_parsed.clone(), to.clone());
        from_parsed.reverse();
        state.mappings.insert(from_parsed, to);

        Ok(())
    }

    #[pyo3(signature = (existing=None))]
    pub fn snapshot(&self, existing: Option<&ChordMapperSnapshot>) -> PyResult<Option<ChordMapperSnapshot>> {
        let mut state = self.state.blocking_lock();
        if let Some(existing) = existing {
            state.mappings = existing.mappings.clone();
            state.chorded_keys = state.mappings.keys().fold(HashSet::new(), |mut acc, e| {
                acc.extend(e.iter().cloned());
                acc
            });
            return Ok(None);
        }
        Ok(Some(ChordMapperSnapshot { mappings: state.mappings.clone() }))
    }

    pub fn link_to(&mut self, target: &PyBound<PyAny>) -> PyResult<()> {
        (self.link.clone() as Arc<dyn LinkSrc>).py_link_to(target)
    }

    pub fn unlink_to(&mut self, py: Python, target: &PyBound<PyAny>) -> PyResult<bool> {
        (self.link.clone() as Arc<dyn LinkSrc>).py_unlink_to(target)
    }

    pub fn unlink_to_all(&mut self) {
        (self.link.clone() as Arc<dyn LinkSrc>).py_unlink_to_all();
    }

    pub fn link_from(&mut self, target: &PyBound<PyAny>) -> PyResult<()> {
        (self.link.clone() as Arc<dyn LinkDst>).py_link_from(target)
    }

    pub fn unlink_from(&mut self, target: &PyBound<PyAny>) -> PyResult<bool> {
        (self.link.clone() as Arc<dyn LinkDst>).py_unlink_from(target)
    }

    pub fn unlink_from_all(&mut self) {
        (self.link.clone() as Arc<dyn LinkDst>).py_unlink_from_all();
    }

    pub fn unlink_all(&mut self) {
        self.unlink_from_all();
        self.unlink_to_all();
    }

    pub fn insert_before(&self, target: &PyBound<PyAny>) -> PyResult<()> {
        (self.link.clone() as Arc<dyn LinkDst>).py_insert_before(target)
    }

    pub fn insert_after(&self, target: &PyBound<PyAny>) -> PyResult<()> {
        (self.link.clone() as Arc<dyn LinkSrc>).py_insert_after(target)
    }

    pub fn name(&self) -> String {
        self.state.blocking_lock().name.clone()
    }

    pub fn next(&self, py: Python) -> Vec<PyObject> {
        self.state.blocking_lock().next.values().map(|v| v.py_object().clone_ref(py).into_any()).collect()
    }

    pub fn prev(&self, py: Python) -> Vec<PyObject> {
        self.state.blocking_lock().prev.values().map(|v| v.py_object().clone_ref(py).into_any()).collect()
    }

    pub fn reset(&mut self) {
        self.state.blocking_lock().mappings.clear();
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence(val.as_str(), Some(&self.state.blocking_lock().transformer))
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

impl Drop for ChordMapper {
    fn drop(&mut self) {
        self.unlink_from_all();
        self.unlink_to_all();
    }
}

#[derive(Clone, derive_new::new)]
pub struct ChordMapperLink {
    id: Uuid,
    ev_tx: Sender<InputEvent>,
    state: Arc<Mutex<State>>,
    #[new(default)]
    py_object: OnceLock<Arc<PyObject>>,
}

impl LinkSrc for ChordMapperLink {
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
    fn clear_next(&self) -> Vec<Arc<dyn LinkDst>> {
        let mut state = self.state.blocking_lock();
        state.next.drain().map(|(_, v)| v).collect()
    }
    fn next(&self) -> Vec<Arc<dyn LinkDst>> {
        let state = self.state.blocking_lock();
        state.next.values().cloned().collect()
    }
}

impl LinkDst for ChordMapperLink {
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
    fn clear_prev(&self) -> Vec<Arc<dyn LinkSrc>> {
        let mut state = self.state.blocking_lock();
        state.prev.drain().map(|(_, v)| v).collect()
    }
    fn prev(&self) -> Vec<Arc<dyn LinkSrc>> {
        let state = self.state.blocking_lock();
        state.prev.values().cloned().collect()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ChordMapperSnapshot {
    mappings: Mappings,
}

async fn handle(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut state = _state.lock().await;

    let ev = match &raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    let _key = Key { event_code: ev.event_code };

    match ev {
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
            state.modifiers.update_from_action(&KeyAction::from_input_ev(&ev));

            // ignore modifiers
            match key {
                KEY_LEFTCTRL | KEY_RIGHTCTRL | KEY_LEFTSHIFT | KEY_RIGHTSHIFT | KEY_LEFTALT | KEY_RIGHTALT
                | KEY_LEFTMETA | KEY_RIGHTMETA => {
                    state.next.send_all(raw_ev);
                    return;
                }
                _ => {}
            };

            match ev.value {
                TYPE_DOWN => {
                    let should_handle = state.chorded_keys.contains(&_key)
                        && state.pressed_keys.iter().all(|x| state.stack.contains(x));

                    state.pressed_keys.insert(_key.clone());

                    if should_handle {
                        state.stack.push(_key.clone());
                        state.interval.take().map(|task| task.abort());

                        if state.stack.len() == 2 {
                            drop(state);
                            handle_cb(_state.clone(), raw_ev).await;
                        } else {
                            let _state = _state.clone();
                            state.interval = Some(tokio::spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                handle_cb(_state.clone(), raw_ev).await;
                            }));
                        }
                    } else {
                        state.interval.take().map(|task| task.abort());

                        let state = &mut *state;
                        for k in state.stack.iter() {
                            state.next.send_all(InputEvent::Raw(k.to_input_ev(TYPE_DOWN)));
                            // state.next.send_all(InputEvent::Raw(k.to_input_ev(TYPE_UP)));
                            // state.ignored_keys.insert(k.clone());
                        }
                        state.stack.clear();

                        state.next.send_all(raw_ev);
                    }
                }
                TYPE_UP => {
                    state.pressed_keys.remove(&_key);
                    state.interval.take().map(|task| task.abort());

                    if let Some(pos) = state.stack.iter().position(|x| x == &_key) {
                        state.stack.remove(pos);

                        if !state.ignored_keys.remove(&_key) {
                            state.next.send_all(InputEvent::Raw(_key.to_input_ev(TYPE_DOWN)));
                            state.next.send_all(raw_ev);
                        }
                    } else {
                        for k in state.stack.iter() {
                            state.next.send_all(InputEvent::Raw(k.to_input_ev(TYPE_DOWN)));
                            state.next.send_all(InputEvent::Raw(k.to_input_ev(TYPE_UP)));
                        }
                        state.stack.clear();

                        if !state.ignored_keys.remove(&_key) {
                            state.next.send_all(raw_ev);
                        }
                    }
                }
                TYPE_REPEAT => {
                    if state.ignored_keys.contains(&_key) {
                        return;
                    }
                    if state.stack.is_empty() {
                        state.next.send_all(raw_ev);
                    };
                }
                _ => unreachable!(),
            };
            return;
        }
        _ => {}
    }

    state.next.send_all(raw_ev);
}

// fired after the chord timeout has passed, submits the keys held on the stack
async fn handle_cb(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut state = _state.lock().await;
    let ev = match raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    if let Some(action) = state.mappings.get(&state.stack).cloned() {
        for k in state.stack.clone().into_iter() {
            state.ignored_keys.insert(k);
        }

        match action {
            RuntimeAction::ActionSequence(seq) => {
                handle_seq2(&seq, &state.modifiers, &state.next, SeqModifierRestoreMode::Default);
            }
            RuntimeAction::PythonCallback(handler) => {
                // TODO pass stack as first arg
                // let args = Some(PythonArgument::String(
                //     match value {
                //         0 => "up",
                //         1 => "down",
                //         2 => "repeat",
                //         _ => unreachable!(),
                //     }
                //     .to_string(),
                // ));

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
    } else {
        // only one key on the stack
        if state.stack.len() == 1 && state.stack[0].event_code == ev.event_code {
            state.next.send_all(InputEvent::Raw(ev.clone()));
        } else {
            // no match, send all buffered keys from stack
            for k in state.stack.iter() {
                state.next.send_all(InputEvent::Raw(k.to_input_ev(1)));
            }
        }
        state.stack.clear();
    }
}
