use self::event_loop::PythonArgument;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::{RuntimeAction, RuntimeKeyAction};
use crate::python::*;
use crate::subscriber::SubscriberNew;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;

#[derive(Default)]
struct MapperState {
    pub modifiers: Arc<KeyModifierState>,
}

impl MapperState {
    fn handle(&mut self, raw_ev: InputEvent, next: Option<&SubscriberNew>, shared_state: &SharedState) {
        let ev = match &raw_ev {
            InputEvent::Raw(ev) => ev,
        };

        match ev {
            // key event
            EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
                let mut from_modifiers = KeyModifierFlags::new();
                from_modifiers.ctrl = self.modifiers.is_ctrl();
                from_modifiers.alt = self.modifiers.is_alt();
                from_modifiers.right_alt = self.modifiers.is_right_alt();
                from_modifiers.shift = self.modifiers.is_shift();
                from_modifiers.meta = self.modifiers.is_meta();

                let from_key_action = KeyActionWithMods {
                    key: Key { event_code: ev.event_code },
                    value: ev.value,
                    modifiers: from_modifiers,
                };

                if let Some(runtime_action) = shared_state.mappings.get(&from_key_action) {
                    match runtime_action {
                        RuntimeAction::ActionSequence(seq) => {
                            let next = match next {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            for action in seq {
                                match action {
                                    RuntimeKeyAction::KeyAction(key_action) => {
                                        let _ = next.send(InputEvent::Raw(key_action.to_input_ev()));
                                    }
                                    RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                        let new_events = release_restore_modifiers(
                                            &self.modifiers,
                                            &from_flags,
                                            &to_flags,
                                            &to_type,
                                        );
                                        for ev in new_events {
                                            let _ = next.send(InputEvent::Raw(ev));
                                        }
                                    }
                                }
                            }
                        }
                        RuntimeAction::PythonCallback(from_modifiers, handler) => {
                            if let Some(next) = next {
                                // always release all trigger mods before running the callback
                                let new_events = release_restore_modifiers(
                                    &self.modifiers,
                                    &from_modifiers,
                                    &KeyModifierFlags::new(),
                                    &TYPE_UP,
                                );
                                for ev in new_events {
                                    let _ = next.send(InputEvent::Raw(ev));
                                }
                            }
                            run_python_handler(&handler, None, ev, &shared_state.transformer, next);
                        }
                        RuntimeAction::NOP => {}
                    }

                    return;
                }

                event_handlers::update_modifiers(&mut self.modifiers, &KeyAction::from_input_ev(&ev));

                if let Some(handler) = shared_state.fallback_handler.as_ref() {
                    let name = match key {
                        KEY_SPACE => "space".to_string(),
                        KEY_TAB => "tab".to_string(),
                        KEY_ENTER => "enter".to_string(),
                        _ => shared_state.transformer.raw_to_utf(key, &*self.modifiers).unwrap_or_else(|| {
                            let name = format!("{key:?}").to_string();
                            name[4..name.len()].to_lowercase()
                        }),
                    };

                    let value = match *value {
                        0 => "up",
                        1 => "down",
                        2 => "repeat",
                        _ => unreachable!(),
                    }
                    .to_string();

                    let args = vec![PythonArgument::String(name), PythonArgument::String(value)];

                    run_python_handler(&handler, Some(args), ev, &shared_state.transformer, next);

                    return;
                }
            }
            // rel/abs event
            EvdevInputEvent { event_code, value, .. }
                if matches!(event_code, EventCode::EV_REL(..)) || matches!(event_code, EventCode::EV_ABS(..)) =>
            {
                let (key, handler) = match event_code {
                    EventCode::EV_REL(key) => (format!("{key:?}").to_string(), &shared_state.relative_handler),
                    EventCode::EV_ABS(key) => (format!("{key:?}").to_string(), &shared_state.absolute_handler),
                    _ => unreachable!(),
                };
                if let Some(handler) = handler.as_ref() {
                    let name = format!("{key:?}");
                    // remove prefix REL_ / ABS_
                    let name = name[4..name.len()].to_string();
                    let args = vec![PythonArgument::String(name), PythonArgument::Number(*value)];
                    run_python_handler(&handler, Some(args), ev, &shared_state.transformer, next);
                    return;
                }
            }
            _ => {}
        }

        if let Some(next) = next {
            let _ = next.send(raw_ev);
        }
    }
}

#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(KeyActionWithMods, RuntimeAction),
}

#[derive(Default)]
struct SharedState {
    id: Arc<Uuid>,
    transformer: Arc<XKBTransformer>,
    mappings: Mappings,
    fallback_handler: Option<PyObject>,
    relative_handler: Option<PyObject>,
    absolute_handler: Option<PyObject>,
}

impl SharedState {}

#[pyclass]
pub struct Mapper {
    pub id: Arc<Uuid>,
    shared_state: Arc<RwLock<SharedState>>,
    transformer: Arc<XKBTransformer>,
    tmp_next: Mutex<Option<SubscriberNew>>,
}

#[pymethods]
impl Mapper {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
            None => HashMap::new(),
        };

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(err_to_py)?;

        let id = Arc::new(Uuid::new_v4());

        let shared_state = Arc::new(RwLock::new(SharedState { id: id.clone(), ..Default::default() }));

        Ok(Self { id, shared_state, transformer, tmp_next: Default::default() })
    }

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        let from = parse_key_action_with_mods(&from, Some(&self.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        if let Ok(to) = to.extract::<String>(py) {
            let to = parse_key_sequence(&to, Some(&self.transformer)).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::KeySequenceParse(err.to_string()),
                ))
            })?;

            self._map_key(from, to)?;
            return Ok(());
        }

        let is_callable = to.as_ref(py).is_callable();

        if is_callable {
            self._map_callback(from, to)?;
            return Ok(());
        }

        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_key(&mut self, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods(&from, Some(&self.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        let to = parse_key_action_with_mods(&to, Some(&self.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'to' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        self._map_key(from, vec![to])?;
        Ok(())
    }

    pub fn map_fallback(&mut self, py: Python, fallback_handler: PyObject) -> PyResult<()> {
        if fallback_handler.as_ref(py).is_callable() {
            self.shared_state.write().unwrap().fallback_handler = Some(fallback_handler);
            return Ok(());
        }
        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_relative(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        if handler.as_ref(py).is_callable() {
            self.shared_state.write().unwrap().relative_handler = Some(handler);
            return Ok(());
        }
        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_absolute(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        if handler.as_ref(py).is_callable() {
            self.shared_state.write().unwrap().absolute_handler = Some(handler);
            return Ok(());
        }
        Err(ApplicationError::NotCallable.into())
    }

    pub fn nop(&mut self, from: String) -> PyResult<()> {
        let from = parse_key_action_with_mods(&from, Some(&self.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            ))
        })?;

        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.shared_state.write().unwrap().mappings.insert(from, RuntimeAction::NOP);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                for value in 0..=2 {
                    let from = KeyActionWithMods::new(from.key, value, from.modifiers);
                    self.shared_state.write().unwrap().mappings.insert(from, RuntimeAction::NOP);
                }
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into());
            }
        }

        Ok(())
    }

    pub fn snapshot(&self, existing: Option<&KeyMapperSnapshot>) -> PyResult<Option<KeyMapperSnapshot>> {
        if let Some(existing) = existing {
            let mut shared_state = self.shared_state.write().unwrap();
            shared_state.mappings = existing.mappings.clone();
            shared_state.fallback_handler = existing.fallback_handler.clone();
            shared_state.relative_handler = existing.relative_handler.clone();
            shared_state.absolute_handler = existing.absolute_handler.clone();
            return Ok(None);
        }

        Ok(Some(KeyMapperSnapshot {
            mappings: self.shared_state.read().unwrap().mappings.clone(),
            fallback_handler: self.shared_state.read().unwrap().fallback_handler.clone(),
            relative_handler: self.shared_state.read().unwrap().relative_handler.clone(),
            absolute_handler: self.shared_state.read().unwrap().absolute_handler.clone(),
        }))
    }
}

impl Mapper {
    pub fn link(&mut self, target: Option<SubscriberNew>) -> PyResult<()> {
        use crate::subscriber::*;

        if let Some(target) = target {
            *self.tmp_next.lock().unwrap() = Some(target);
        }
        Ok(())
    }

    pub fn subscribe(&self) -> SubscriberNew {
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::unbounded_channel::<InputEvent>();
        let next = self.tmp_next.lock().unwrap().take();

        let _shared_state = self.shared_state.clone();
        get_runtime().spawn(async move {
            let mut state = MapperState::default();
            loop {
                let ev = ev_rx.recv().await.unwrap();
                let shared_state = _shared_state.read().unwrap();
                state.handle(ev, next.as_ref(), &shared_state);
            }
        });

        ev_tx.clone()
    }

    fn _map_callback(&mut self, from: ParsedKeyAction, to: PyObject) -> PyResult<()> {
        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.shared_state
                    .write()
                    .unwrap()
                    .mappings
                    .insert(from, RuntimeAction::PythonCallback(from.modifiers, to));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                self.shared_state
                    .write()
                    .unwrap()
                    .mappings
                    .insert(from.to_key_action(1), RuntimeAction::PythonCallback(from.modifiers, to));
                self.shared_state.write().unwrap().mappings.insert(from.to_key_action(0), RuntimeAction::NOP);
                self.shared_state.write().unwrap().mappings.insert(from.to_key_action(2), RuntimeAction::NOP);
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into());
            }
        }

        Ok(())
    }

    fn _map_key(&mut self, from: ParsedKeyAction, mut to: Vec<ParsedKeyAction>) -> PyResult<()> {
        match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    match to {
                        // key action to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mapping = map_action_to_click(&from, &to);
                            self.shared_state.write().unwrap().mappings.insert(mapping.0, mapping.1);
                        }
                        // key action to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mapping = map_action_to_action(&from, &to);
                            self.shared_state.write().unwrap().mappings.insert(mapping.0, mapping.1);
                        }
                        // key action to action
                        ParsedKeyAction::Action(to) => {
                            let mapping = map_action_to_action(&from, &to.to_key_action_with_mods(Default::default()));
                            self.shared_state.write().unwrap().mappings.insert(mapping.0, mapping.1);
                        }
                    }
                    return Ok(());
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                self.shared_state.write().unwrap().mappings.insert(mapping.0, mapping.1);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    match to.remove(0) {
                        // click to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mappings = map_click_to_click(&from, &to);

                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                self.shared_state.write().unwrap().mappings.insert(from, to);
                            });
                        }
                        // click to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mappings = map_click_to_action(&from, &to);
                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                self.shared_state.write().unwrap().mappings.insert(from, to);
                            });
                        }
                        // click to action
                        ParsedKeyAction::Action(to) => {
                            let to = to.to_key_action_with_mods(Default::default());
                            let mappings = map_click_to_action(&from, &to);
                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                self.shared_state.write().unwrap().mappings.insert(from, to);
                            });
                        }
                    };
                    return Ok(());
                }

                // click to seq
                let mappings = map_click_to_seq(from, to);
                IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                    self.shared_state.write().unwrap().mappings.insert(from, to);
                });
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into());
            }
        }

        Ok(())
    }
}

#[pyclass]
pub struct KeyMapperSnapshot {
    mappings: Mappings,
    fallback_handler: Option<PyObject>,
    relative_handler: Option<PyObject>,
    absolute_handler: Option<PyObject>,
}
