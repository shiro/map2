use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeKeyAction;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use nom::Slice;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use ApplicationError::TooManyEvents;
type Mappings = HashMap<Vec<Key>, RuntimeAction>;

#[derive(Default)]
struct State {
    transformer: Arc<XKBTransformer>,
    prev: HashMap<Uuid, Arc<dyn LinkSrc>>,
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
    mappings: Mappings,
    chorded_keys: HashSet<Key>,
    modifiers: Arc<KeyModifierState>,
    stack: Vec<Key>,
    ignored_keys: HashSet<Key>,
    pressed_keys: HashSet<Key>,
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
    pub fn new(kwargs: Option<pyo3::Bound<PyDict>>) -> PyResult<Self> {
        let options: HashMap<String, Bound<PyAny>> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new(),
        };

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(err_to_py)?;

        let id = Uuid::new_v4();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::channel(64);
        let state = Arc::new(Mutex::new(State { transformer, ..Default::default() }));
        let link = Arc::new(ChordMapperLink { id, ev_tx: ev_tx.clone(), state: state.clone() });

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

        Ok(Self { id, link, ev_tx, state })
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
            RuntimeAction::PythonCallback(Default::default(), Arc::new(to))
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

            let mut to: Vec<RuntimeKeyAction> =
                to.to_key_actions().into_iter().map(|action| RuntimeKeyAction::KeyAction(action)).collect();
            RuntimeAction::ActionSequence(to)
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

    pub fn link_to(&mut self, target: &pyo3::Bound<PyAny>) -> PyResult<()> {
        let mut target = node_to_link_dst(target).unwrap();
        target.link_from(self.link.clone());
        self.link.link_to(target);
        Ok(())
    }

    pub fn unlink_to(&mut self, py: Python, target: &pyo3::Bound<PyAny>) -> PyResult<bool> {
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

    pub fn link_from(&mut self, target: &pyo3::Bound<PyAny>) -> PyResult<()> {
        let target = node_to_link_src(target).ok_or_else(|| PyRuntimeError::new_err("expected a source node"))?;
        target.link_to(self.link.clone());
        self.link.link_from(target);
        Ok(())
    }

    pub fn unlink_from(&mut self, target: &pyo3::Bound<PyAny>) -> PyResult<bool> {
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
        "".to_string()
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence(val.as_str(), Some(&self.state.blocking_lock().transformer))
            .map_err(|err| ApplicationError::KeySequenceParse(err.to_string()).into_py())?
            .to_key_actions();
        for action in actions {
            self.ev_tx.try_send(InputEvent::Raw(action.to_input_ev())).expect(&TooManyEvents.to_string());
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

#[derive(Clone)]
pub struct ChordMapperLink {
    id: Uuid,
    ev_tx: Sender<InputEvent>,
    state: Arc<Mutex<State>>,
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
            event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));

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
                        if state.next.is_empty() {
                            return;
                        };

                        if let Some(task) = state.interval.take() {
                            task.abort();
                        };

                        fn foo(v: &mut State) -> (&Vec<Key>, &mut HashSet<Key>) {
                            (&v.stack, &mut v.ignored_keys)
                        }

                        let state = &mut *state;
                        for k in state.stack.iter() {
                            state.next.send_all(InputEvent::Raw(k.to_input_ev(TYPE_DOWN)));
                            state.next.send_all(InputEvent::Raw(k.to_input_ev(TYPE_UP)));
                            state.ignored_keys.insert(k.clone());
                        }
                        state.stack.clear();

                        state.next.send_all(raw_ev);
                    }
                }
                TYPE_UP => {
                    state.pressed_keys.remove(&_key);
                    if state.next.is_empty() {
                        return;
                    };

                    if let Some(task) = state.interval.take() {
                        task.abort();
                    };

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
        }
        _ => {}
    }
}

// fired after the chord timeout has passed, submits the keys held on the stack
async fn handle_cb(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut _state = _state.lock().await;
    let state = &mut *_state;
    let ev = match raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    if let Some(task) = state.interval.take() {
        task.abort();
    };

    if let Some(action) = state.mappings.get(&state.stack) {
        for k in state.stack.iter().cloned() {
            state.ignored_keys.insert(k);
        }

        match action {
            RuntimeAction::ActionSequence(seq) => {
                for action in seq {
                    match action {
                        RuntimeKeyAction::KeyAction(key_action) => {
                            state.next.send_all(InputEvent::Raw(key_action.to_input_ev()));
                        }
                        RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                            let new_events =
                                release_restore_modifiers(&state.modifiers, &from_flags, &to_flags, &to_type);
                            if !state.next.is_empty() {
                                for ev in new_events {
                                    state.next.send_all(InputEvent::Raw(ev));
                                }
                            }
                        }
                    }
                }
            }
            RuntimeAction::PythonCallback(from_modifiers, handler) => {
                if !state.next.is_empty() {
                    // always release all trigger mods before running the callback
                    let new_events = release_restore_modifiers(
                        &state.modifiers,
                        &from_modifiers,
                        &KeyModifierFlags::new(),
                        &TYPE_UP,
                    );
                    for ev in new_events {
                        state.next.send_all(InputEvent::Raw(ev));
                    }
                }

                let handler = handler.clone();
                let transformer = state.transformer.clone();
                let next = state.next.values().cloned().collect();
                drop(state);
                drop(_state);
                run_python_handler(handler, None, ev.clone(), transformer, next).await;
            }
            RuntimeAction::NOP => {}
        }
    } else {
        if state.next.is_empty() {
            return;
        };

        // only one key on the stack
        if state.stack.len() == 1 && state.stack[0].event_code == ev.event_code {
            state.next.send_all(InputEvent::Raw(ev.clone()));
        } else {
            // no match, send all buffered keys from stack
            for k in state.stack.iter() {
                state.next.send_all(InputEvent::Raw(k.to_input_ev(1)));
                state.next.send_all(InputEvent::Raw(k.to_input_ev(0)));
            }
        }
        state.stack.clear();
    }
}
