use self::event_loop::PythonArgument;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::{RuntimeAction, RuntimeKeyAction};
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use evdev_rs::enums::EV_KEY;
use futures::executor::block_on;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use ApplicationError::TooManyEvents;

#[derive(derive_new::new)]
struct State {
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
    fallback_handler: Option<Arc<PyObject>>,
    #[new(default)]
    modifiers: Arc<KeyModifierState>,
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
    pub fn new(key: String, kwargs: Option<pyo3::Bound<PyDict>>) -> PyResult<Self> {
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

        let key = parse_key(&key, Some(&transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!("failed to parse key '{}'", ApplicationError::KeyParse(err.to_string()),))
        })?;

        let id = Uuid::new_v4();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::channel(64);
        let state = Arc::new(Mutex::new(State::new(transformer, key)));
        let link = Arc::new(MapperLink { id, ev_tx: ev_tx.clone(), state: state.clone() });

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

    pub fn map_fallback(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.bind(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.fallback_handler = Some(Arc::new(handler));
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
            state.fallback_handler = existing.fallback_handler.clone();
            return Ok(None);
        }
        Ok(Some(ModifierMapperSnapshot {
            mappings: state.mappings.clone(),
            fallback_handler: state.fallback_handler.clone(),
        }))
    }

    pub fn link_to(&mut self, target: &pyo3::Bound<PyAny>) -> PyResult<()> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
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
                state.mappings.insert(from, RuntimeAction::PythonCallback(from.modifiers, to));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                state.mappings.insert(from.to_key_action(1), RuntimeAction::PythonCallback(from.modifiers, to));
                state.mappings.insert(from.to_key_action(0), RuntimeAction::NOP);
                state.mappings.insert(from.to_key_action(2), RuntimeAction::NOP);
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
        self.unlink_from_all();
        self.unlink_to_all();
    }
}

#[derive(Clone)]
pub struct MapperLink {
    id: Uuid,
    ev_tx: Sender<InputEvent>,
    state: Arc<Mutex<State>>,
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
}

#[pyclass]
pub struct ModifierMapperSnapshot {
    mappings: Mappings,
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
            let surpressed = state.surpressed;
            state.surpressed = false;

            match value {
                0 => {
                    state.active = false;

                    if !surpressed {
                        state.next.send_all(InputEvent::Raw(state.key.to_input_ev(1)));
                        state.next.send_all(InputEvent::Raw(state.key.to_input_ev(0)));
                    }

                    // TODO order
                    for key_raw in state.down_keys.clone().iter() {
                        let key = Key::from(key_raw.clone());
                        let mut from_modifiers = KeyModifierFlags::new();
                        from_modifiers.ctrl = state.modifiers.is_ctrl();
                        from_modifiers.alt = state.modifiers.is_alt();
                        from_modifiers.right_alt = state.modifiers.is_right_alt();
                        from_modifiers.shift = state.modifiers.is_shift();
                        from_modifiers.meta = state.modifiers.is_meta();
                        let mut from_key_action =
                            KeyActionWithMods { key: key.clone(), value: 1, modifiers: from_modifiers };

                        // skip unrelated
                        if state.mappings.get(&from_key_action).is_none() {
                            continue;
                        }

                        // trigger up action on mapped key
                        from_key_action.value = 0;
                        if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                            if let Some(handler) = handle_action(&state, key_raw, *value, runtime_action) {
                                let args = Some(make_args(&state, key_raw, *value));
                                drop(ev);
                                let transformer = state.transformer.clone();
                                let next = state.next.values().cloned().collect();
                                drop(state);
                                run_python_handler(handler, args, ev.clone(), transformer, next).await;
                                state = _state.lock().await;
                            };
                        }
                    }

                    let mut from_key_action =
                        KeyActionWithMods { key: state.key.clone(), value: 0, modifiers: KeyModifierFlags::new() };
                    if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                        if let Some(handler) = handle_action(&state, key, *value, runtime_action) {
                            let args = Some(make_args(&state, key, *value));
                            drop(ev);
                            let ev = match raw_ev {
                                InputEvent::Raw(ev) => ev,
                            };
                            let transformer = state.transformer.clone();
                            let next = state.next.values().cloned().collect();
                            drop(state);
                            run_python_handler(handler, args, ev, transformer, next).await;
                            state = _state.lock().await;
                        };
                    }

                    // press down on held keys after modifier is released
                    for key in state.down_keys.iter() {
                        let key = Key::from(key.clone());
                        state.next.send_all(InputEvent::Raw(key.to_input_ev(1)));
                    }
                    state.down_keys.clear();
                }
                1 => {
                    state.active = true;
                    // TODO on down
                    let mut from_key_action =
                        KeyActionWithMods { key: state.key.clone(), value: 1, modifiers: KeyModifierFlags::new() };
                    if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                        if let Some(handler) = handle_action(&state, key, *value, runtime_action) {
                            let args = Some(make_args(&state, key, *value));
                            drop(ev);
                            let ev = match raw_ev {
                                InputEvent::Raw(ev) => ev,
                            };
                            let transformer = state.transformer.clone();
                            let next = state.next.values().cloned().collect();
                            drop(state);
                            run_python_handler(handler, args, ev, transformer, next).await;
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

    if !state.active {
        state.next.send_all(raw_ev);
        return;
    }

    match ev {
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
            state.surpressed = true;

            if *value == 1 {
                state.down_keys.insert(key.clone());
            }
            if *value == 0 {
                state.down_keys.remove(key);
            }

            let mut from_modifiers = KeyModifierFlags::new();
            from_modifiers.ctrl = state.modifiers.is_ctrl();
            from_modifiers.alt = state.modifiers.is_alt();
            from_modifiers.right_alt = state.modifiers.is_right_alt();
            from_modifiers.shift = state.modifiers.is_shift();
            from_modifiers.meta = state.modifiers.is_meta();

            let from_key_action = KeyActionWithMods {
                key: Key { event_code: ev.event_code },
                value: ev.value,
                modifiers: from_modifiers,
            };

            if let Some(runtime_action) = state.mappings.get(&from_key_action) {
                if let Some(handler) = handle_action(&state, key, *value, runtime_action) {
                    let args = Some(make_args(&state, key, *value));
                    drop(ev);
                    let ev = match raw_ev {
                        InputEvent::Raw(ev) => ev,
                    };
                    let transformer = state.transformer.clone();
                    let next = state.next.values().cloned().collect();
                    drop(state);
                    run_python_handler(handler, args, ev, transformer, next).await;
                };

                return;
            }

            event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));

            if let Some(handler) = state.fallback_handler.as_ref() {
                let args = Some(make_args(&state, key, *value));
                drop(ev);
                let ev = match raw_ev {
                    InputEvent::Raw(ev) => ev,
                };
                let handler = handler.clone();
                let transformer = state.transformer.clone();
                let next = state.next.values().cloned().collect();
                drop(state);
                run_python_handler(handler, args, ev, transformer, next).await;
                return;
            }
        }
        _ => {}
    }

    state.next.send_all(raw_ev);
}

fn handle_action(state: &State, key: &EV_KEY, value: i32, runtime_action: &RuntimeAction) -> Option<Arc<PyObject>> {
    match runtime_action {
        RuntimeAction::ActionSequence(seq) => {
            for action in seq {
                match action {
                    RuntimeKeyAction::KeyAction(key_action) => {
                        let _ = state.next.send_all(InputEvent::Raw(key_action.to_input_ev()));
                    }
                    RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                        let new_events = release_restore_modifiers(&state.modifiers, &from_flags, &to_flags, &to_type);
                        for ev in new_events {
                            state.next.send_all(InputEvent::Raw(ev));
                        }
                    }
                }
            }
        }
        RuntimeAction::PythonCallback(from_modifiers, handler) => {
            if !state.next.is_empty() {
                // always release all trigger mods before running the callback
                let new_events =
                    release_restore_modifiers(&state.modifiers, &from_modifiers, &KeyModifierFlags::new(), &TYPE_UP);
                new_events.iter().cloned().for_each(|ev| state.next.send_all(InputEvent::Raw(ev)));
            }

            return Some(handler.clone());
        }
        _ => {}
    };
    None
}

fn make_args(state: &State, key: &EV_KEY, value: i32) -> Vec<PythonArgument> {
    let name = match key {
        KEY_SPACE => "space".to_string(),
        KEY_TAB => "tab".to_string(),
        KEY_ENTER => "enter".to_string(),
        _ => state.transformer.raw_to_utf(key, &*state.modifiers).unwrap_or_else(|| {
            let name = format!("{key:?}").to_string();
            name[4..name.len()].to_lowercase()
        }),
    };

    let value = match value {
        0 => "up",
        1 => "down",
        2 => "repeat",
        _ => unreachable!(),
    }
    .to_string();

    vec![PythonArgument::String(name), PythonArgument::String(value)]
}
