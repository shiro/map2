use self::event_loop::PythonArgument;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeAction;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use evdev_rs::enums::EV_KEY;
use futures::executor::block_on;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, MutexGuard};

use ApplicationError::TooManyEvents;

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
    key: Key,
    #[new(default)]
    active: bool,
    #[new(default)]
    surpressed: bool,
    #[new(default)]
    down_keys: HashSet<EV_KEY>,
    #[new(default)]
    ignored_keys: HashSet<EV_KEY>,
    #[new(default)]
    click_action: Option<RuntimeAction>,
    #[new(default)]
    fallback_handler: Option<Arc<PyObject>>,
    #[new(default)]
    modifiers: KeyModifierFlags,
}

#[pyclass]
pub struct ModifierMapper {
    pub id: Uuid,
    pub link: Arc<MapperLink>,
    ev_tx: tokio::sync::mpsc::Sender<InputEvent>,
    state: Arc<Mutex<State>>,
}

#[pymethods]
impl ModifierMapper {
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
            .unwrap_or(format!("modifier mapper {}", node_util::get_id_and_incremen(&ID_COUNTER)))
            .to_string();
        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(err_to_py)?;

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

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        let from = parse_key_action_with_mods(&from, Some(&state.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        if let Ok(to) = to.extract::<String>(py) {
            let to = parse_key_sequence(&to, Some(&state.transformer)).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::KeySequenceParse(err.to_string()),
                ))
            })?;

            drop(state);
            self._map_key(from, to)?;
            return Ok(());
        }

        let is_callable = to.bind(py).is_callable();

        if is_callable {
            drop(state);
            self._map_callback(from, to)?;
            return Ok(());
        }

        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_key(&mut self, from: String, to: String) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        let from = parse_key_action_with_mods(&from, Some(&state.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        let to = parse_key_action_with_mods(&to, Some(&state.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'to' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        drop(state);
        self._map_key(from, vec![to])?;
        Ok(())
    }

    pub fn map_click(&mut self, py: Python, to: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();

        if let Ok(to) = to.extract::<String>(py) {
            let to = parse_key_sequence(&to, Some(&state.transformer)).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::KeySequenceParse(err.to_string()),
                ))
            })?;

            // let mut seq: Vec<RuntimeKeyActionDepr> =
            // to.to_key_actions().into_iter().map(|action| RuntimeKeyActionDepr::KeyAction(action)).collect();
            state.click_action = Some(RuntimeAction::ActionSequence(to.to_key_actions_with_mods()));
            // TODO release resotre here needed?

            return Ok(());
        }

        let is_callable = to.bind(py).is_callable();

        if is_callable {
            // TODO check flags field (1)
            state.click_action = Some(RuntimeAction::PythonCallback(Arc::new(to)));
            return Ok(());
        }

        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_fallback(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.bind(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.fallback_handler = Some(Arc::new(handler));
        Ok(())
    }

    pub fn nop(&mut self, from: String) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        let from = parse_key_action_with_mods(&from, Some(&state.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        match from {
            ParsedKeyAction::KeyAction(from) => {
                state.mappings.insert(from, RuntimeAction::NOP);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                for value in 0..=2 {
                    let from = KeyActionWithMods::new(from.key, value, from.modifiers);
                    state.mappings.insert(from, RuntimeAction::NOP);
                }
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into_py());
            }
        }
        Ok(())
    }

    pub fn snapshot(
        &self,
        py: Python,
        existing: Option<&ModifierMapperSnapshot>,
    ) -> PyResult<Option<ModifierMapperSnapshot>> {
        let mut state = self.state.blocking_lock();
        if let Some(existing) = existing {
            state.mappings = existing.mappings.clone();
            state.click_action = existing.click_action.clone();
            state.fallback_handler = existing.fallback_handler.clone();
            return Ok(None);
        }
        Ok(Some(ModifierMapperSnapshot {
            mappings: state.mappings.clone(),
            click_action: state.click_action.clone(),
            fallback_handler: state.fallback_handler.clone(),
        }))
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
            self.ev_tx.try_send(InputEvent::Raw(action.to_input_ev())).expect(&TooManyEvents.to_string());
        }
        Ok(())
    }
}

impl ModifierMapper {
    fn _map_callback(&mut self, from: ParsedKeyAction, to: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        let to = Arc::new(to);
        match from {
            ParsedKeyAction::KeyAction(from) => {
                state.mappings.insert(from, RuntimeAction::PythonCallback(to));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                state.mappings.insert(from.to_key_action_with_mods(1), RuntimeAction::PythonCallback(to));
                state.mappings.insert(from.to_key_action_with_mods(0), RuntimeAction::NOP);
                state.mappings.insert(from.to_key_action_with_mods(2), RuntimeAction::NOP);
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into());
            }
        }

        Ok(())
    }

    fn _map_key(&mut self, from: ParsedKeyAction, mut to: Vec<ParsedKeyAction>) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    match to {
                        // key action to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mapping = map_action_to_click(&from, &to);
                            state.mappings.insert(mapping.0, mapping.1);
                        }
                        // key action to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mapping = map_action_to_action(&from, &to);
                            state.mappings.insert(mapping.0, mapping.1);
                        }
                        // key action to action
                        ParsedKeyAction::Action(to) => {
                            let mapping = map_action_to_action(&from, &to.to_key_action_with_mods(Default::default()));
                            state.mappings.insert(mapping.0, mapping.1);
                        }
                    }
                    return Ok(());
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                state.mappings.insert(mapping.0, mapping.1);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    match to.remove(0) {
                        // click to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mappings = map_click_to_click(&from, &to);

                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                state.mappings.insert(from, to);
                            });
                        }
                        // click to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mappings = map_click_to_action(&from, &to);
                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                state.mappings.insert(from, to);
                            });
                        }
                        // click to action
                        ParsedKeyAction::Action(to) => {
                            let to = to.to_key_action_with_mods(Default::default());
                            let mappings = map_click_to_action(&from, &to);
                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                state.mappings.insert(from, to);
                            });
                        }
                    };
                    return Ok(());
                }

                // click to seq
                let mappings = map_click_to_seq(from, to);
                IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                    state.mappings.insert(from, to);
                });
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into());
            }
        }
        Ok(())
    }
}

impl Drop for ModifierMapper {
    fn drop(&mut self) {
        println!("bye");
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

#[pyclass]
pub struct ModifierMapperSnapshot {
    mappings: Mappings,
    click_action: Option<RuntimeAction>,
    fallback_handler: Option<Arc<PyObject>>,
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
            let event_code = EventCode::EV_KEY(*key);
            let surpressed = state.surpressed;
            state.surpressed = false;

            match value {
                0 => {
                    state.active = false;

                    // release all other held keys
                    // TODO order
                    for key_raw in state.down_keys.clone().iter() {
                        if state.ignored_keys.contains(key_raw) {
                            continue;
                        }

                        let key = Key::from(key_raw.clone());
                        let mut from_key_action =
                            KeyActionWithMods { key: key.clone(), value: 1, modifiers: state.modifiers };

                        // skip unrelated
                        if state.mappings.get(&from_key_action).is_none() {
                            continue;
                        }

                        // trigger up action on mapped key
                        from_key_action.value = 0;
                        if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                            match runtime_action {
                                RuntimeAction::ActionSequence(seq) => {
                                    let mode = get_mode(&state.mappings, &from_key_action, seq);
                                    handle_seq2(seq, &state.modifiers, &state.next, mode);
                                }
                                RuntimeAction::PythonCallback(handler) => {
                                    handle_callback(
                                        &ev,
                                        handler.clone(),
                                        Some(python_callback_args(
                                            &event_code,
                                            &state.modifiers,
                                            *value,
                                            &state.transformer,
                                        )),
                                        state.transformer.clone(),
                                        &state.modifiers.clone(),
                                        state.next.values().cloned().collect(),
                                        state,
                                    )
                                    .await;
                                    state = _state.lock().await;
                                }
                                _ => {}
                            };
                            continue;
                        }

                        if let Some(handler) = state.fallback_handler.as_ref() {
                            handle_callback(
                                &ev,
                                handler.clone(),
                                Some(python_callback_args(&key.event_code, &Default::default(), 0, &state.transformer)),
                                state.transformer.clone(),
                                &state.modifiers.clone(),
                                state.next.values().cloned().collect(),
                                state,
                            )
                            .await;
                            return;
                        }
                    }

                    // up action on modifier
                    let mut from_key_action =
                        KeyActionWithMods { key: state.key.clone(), value: 0, modifiers: KeyModifierFlags::new() };

                    if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                        match runtime_action {
                            RuntimeAction::ActionSequence(seq) => {
                                let mode = get_mode(&state.mappings, &from_key_action, seq);
                                handle_seq2(seq, &state.modifiers, &state.next, mode);
                            }
                            RuntimeAction::PythonCallback(handler) => {
                                handle_callback(
                                    &ev,
                                    handler.clone(),
                                    Some(python_callback_args(
                                        &event_code,
                                        &state.modifiers,
                                        *value,
                                        &state.transformer,
                                    )),
                                    state.transformer.clone(),
                                    &state.modifiers.clone(),
                                    state.next.values().cloned().collect(),
                                    state,
                                )
                                .await;
                                state = _state.lock().await;
                            }
                            _ => {}
                        };
                    }

                    // other keys were not pressed pressed
                    if !surpressed {
                        if let Some(runtime_action) = &state.click_action {
                            match runtime_action {
                                RuntimeAction::ActionSequence(seq) => {
                                    let mode = get_mode(&state.mappings, &from_key_action, seq);
                                    handle_seq2(seq, &state.modifiers, &state.next, mode);
                                }
                                RuntimeAction::PythonCallback(handler) => {
                                    handle_callback(
                                        &ev,
                                        handler.clone(),
                                        Some(python_callback_args(
                                            &event_code,
                                            &state.modifiers,
                                            *value,
                                            &state.transformer,
                                        )),
                                        state.transformer.clone(),
                                        &state.modifiers.clone(),
                                        state.next.values().cloned().collect(),
                                        state,
                                    )
                                    .await;
                                    state = _state.lock().await;
                                }
                                _ => {}
                            };
                        } else {
                            state.next.send_all(InputEvent::Raw(state.key.to_input_ev(1)));
                            state.next.send_all(InputEvent::Raw(state.key.to_input_ev(0)));
                        }
                    }

                    // enabling this is more correct, but annoying
                    // press down other held keys after modifier is released
                    // for key in state.down_keys.iter() {
                    //     let key = Key::from(key.clone());
                    //     state.next.send_all(InputEvent::Raw(key.to_input_ev(1)));
                    // }
                    state.down_keys.clear();
                }
                1 => {
                    state.active = true;

                    let mut from_key_action =
                        KeyActionWithMods { key: state.key.clone(), value: 1, modifiers: KeyModifierFlags::new() };

                    if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                        match runtime_action {
                            RuntimeAction::ActionSequence(seq) => {
                                let mode = get_mode(&state.mappings, &from_key_action, seq);
                                handle_seq2(seq, &state.modifiers, &state.next, mode);
                            }
                            RuntimeAction::PythonCallback(handler) => {
                                handle_callback(
                                    &ev,
                                    handler.clone(),
                                    Some(python_callback_args(
                                        &event_code,
                                        &state.modifiers,
                                        *value,
                                        &state.transformer,
                                    )),
                                    state.transformer.clone(),
                                    &state.modifiers.clone(),
                                    state.next.values().cloned().collect(),
                                    state,
                                )
                                .await;
                            }
                            _ => {}
                        };
                    }
                }
                2 => {
                    // TODO on repeat
                }
                _ => unreachable!(),
            }
            return;
        }
        _ => {}
    };

    match ev {
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
            let event_code = EventCode::EV_KEY(*key);

            if *value == 1 {
                state.surpressed = true;
                state.down_keys.insert(key.clone());
            }
            if *value == 0 {
                state.down_keys.remove(key);
            }

            // don't process keys pressed and held before the mod was pressed
            if state.ignored_keys.contains(key) {
                if *value == 0 {
                    state.ignored_keys.remove(key);
                }
                state.next.send_all(raw_ev);
                return;
            }
            if !state.active {
                if *value == 1 {
                    state.ignored_keys.insert(key.clone());
                }
                state.next.send_all(raw_ev);
                return;
            }

            let from_key_action = KeyActionWithMods {
                key: Key { event_code: ev.event_code },
                value: ev.value,
                modifiers: state.modifiers,
            };

            if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                // handle mapping if it exists
                match runtime_action {
                    RuntimeAction::ActionSequence(seq) => {
                        let mode = get_mode(&state.mappings, &from_key_action, seq);
                        handle_seq2(seq, &state.modifiers, &state.next, mode);
                    }
                    RuntimeAction::PythonCallback(handler) => {
                        handle_callback(
                            &ev,
                            handler.clone(),
                            Some(python_callback_args(&event_code, &state.modifiers, *value, &state.transformer)),
                            state.transformer.clone(),
                            &state.modifiers.clone(),
                            state.next.values().cloned().collect(),
                            state,
                        )
                        .await;
                        return;
                    }
                    _ => {}
                };

                return;
            }

            // event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));
            state.modifiers.update_from_action(&KeyAction::from_input_ev(&ev));

            if let Some(handler) = state.fallback_handler.as_ref() {
                handle_callback(
                    &ev,
                    handler.clone(),
                    Some(python_callback_args(&event_code, &state.modifiers, *value, &state.transformer)),
                    state.transformer.clone(),
                    &state.modifiers.clone(),
                    state.next.values().cloned().collect(),
                    state,
                )
                .await;
                return;
            }
        }
        _ => {}
    }

    state.next.send_all(raw_ev);
}

async fn handle_action_python_callback<'a>(
    ev: &EvdevInputEvent,
    mut state: MutexGuard<'a, State>,
    handler: &Arc<PyObject>,
    args: Option<Vec<PythonArgument>>,
) {
    let transformer = state.transformer.clone();
    let next = state.next.values().cloned().collect();
    drop(state);
    run_python_handler(handler.clone(), args, ev.clone(), transformer, next).await;
}
