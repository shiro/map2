use self::event_loop::PythonArgument;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::{RuntimeAction, RuntimeKeyAction};
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use futures::executor::block_on;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use ApplicationError::TooManyEvents;

#[derive(Default)]
struct State {
    transformer: Arc<XKBTransformer>,
    prev: HashMap<Uuid, Arc<dyn LinkSrc>>,
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
    mappings: Mappings,
    fallback_handler: Option<Arc<PyObject>>,
    relative_handler: Option<Arc<PyObject>>,
    absolute_handler: Option<Arc<PyObject>>,
    modifiers: Arc<KeyModifierState>,
}

#[pyclass]
pub struct Mapper {
    pub id: Uuid,
    pub link: Arc<MapperLink>,
    ev_tx: tokio::sync::mpsc::Sender<InputEvent>,
    state: Arc<Mutex<State>>,
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

        let id = Uuid::new_v4();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::channel(32);
        let state = Arc::new(Mutex::new(State { transformer, ..Default::default() }));
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

        let is_callable = to.as_ref(py).is_callable();

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
        if !handler.as_ref(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.fallback_handler = Some(Arc::new(handler));
        Ok(())
    }

    pub fn map_relative(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.as_ref(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.relative_handler = Some(Arc::new(handler));
        Ok(())
    }

    pub fn map_absolute(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.as_ref(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.absolute_handler = Some(Arc::new(handler));
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

    pub fn snapshot(&self, py: Python, existing: Option<&KeyMapperSnapshot>) -> PyResult<Option<KeyMapperSnapshot>> {
        let mut state = self.state.blocking_lock();
        if let Some(existing) = existing {
            state.mappings = existing.mappings.clone();
            state.fallback_handler = existing.fallback_handler.clone();
            state.relative_handler = existing.relative_handler.clone();
            state.absolute_handler = existing.absolute_handler.clone();
            return Ok(None);
        }
        Ok(Some(KeyMapperSnapshot {
            mappings: state.mappings.clone(),
            fallback_handler: state.fallback_handler.clone(),
            relative_handler: state.relative_handler.clone(),
            absolute_handler: state.absolute_handler.clone(),
        }))
    }

    pub fn link_to(&mut self, target: &PyAny) -> PyResult<()> {
        let target = node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a destination node"))?;
        target.link_from(self.link.clone());
        self.link.link_to(target);
        Ok(())
    }

    pub fn unlink_to(&mut self, py: Python, target: &PyAny) -> PyResult<bool> {
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

    pub fn unlink_from(&mut self, target: &PyAny) -> PyResult<bool> {
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

impl Mapper {
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

impl Drop for Mapper {
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
    fn send(&self, ev: InputEvent) {
        self.ev_tx.try_send(ev).expect(&ApplicationError::TooManyEvents.to_string());
    }
}

#[pyclass]
pub struct KeyMapperSnapshot {
    mappings: Mappings,
    fallback_handler: Option<Arc<PyObject>>,
    relative_handler: Option<Arc<PyObject>>,
    absolute_handler: Option<Arc<PyObject>>,
}

async fn handle(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut state = _state.lock().await;
    let ev = match &raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    match ev {
        // key event
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
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
                match runtime_action {
                    RuntimeAction::ActionSequence(seq) => {
                        for action in seq {
                            match action {
                                RuntimeKeyAction::KeyAction(key_action) => {
                                    let _ = state.next.send_all(InputEvent::Raw(key_action.to_input_ev()));
                                }
                                RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                    let new_events =
                                        release_restore_modifiers(&state.modifiers, &from_flags, &to_flags, &to_type);
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
                            let new_events = release_restore_modifiers(
                                &state.modifiers,
                                &from_modifiers,
                                &KeyModifierFlags::new(),
                                &TYPE_UP,
                            );
                            new_events.iter().cloned().for_each(|ev| state.next.send_all(InputEvent::Raw(ev)));
                        }

                        drop(ev);
                        let ev = match raw_ev {
                            InputEvent::Raw(ev) => ev,
                        };
                        run_python_handler(
                            handler.clone(),
                            None,
                            ev,
                            state.transformer.clone(),
                            state.next.values().cloned().collect(),
                        )
                        .await;
                    }
                    RuntimeAction::NOP => {}
                }

                return;
            }

            event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));

            if let Some(handler) = state.fallback_handler.as_ref() {
                let name = match key {
                    KEY_SPACE => "space".to_string(),
                    KEY_TAB => "tab".to_string(),
                    KEY_ENTER => "enter".to_string(),
                    _ => state.transformer.raw_to_utf(key, &*state.modifiers).unwrap_or_else(|| {
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
                drop(ev);
                let ev = match raw_ev {
                    InputEvent::Raw(ev) => ev,
                };
                run_python_handler(
                    handler.clone(),
                    Some(args),
                    ev,
                    state.transformer.clone(),
                    state.next.values().cloned().collect(),
                )
                .await;
                return;
            }
        }
        // rel/abs event
        EvdevInputEvent { event_code, value, .. }
            if matches!(event_code, EventCode::EV_REL(..)) || matches!(event_code, EventCode::EV_ABS(..)) =>
        {
            let (key, handler) = match event_code {
                EventCode::EV_REL(key) => (format!("{key:?}").to_string(), &state.relative_handler),
                EventCode::EV_ABS(key) => (format!("{key:?}").to_string(), &state.absolute_handler),
                _ => unreachable!(),
            };
            if let Some(handler) = handler.as_ref() {
                let name = format!("{key:?}");
                // remove prefix REL_ / ABS_
                // let name = name[5..name.len() - 1].to_string();
                let name = name[1..name.len() - 1].to_lowercase();
                let args = vec![PythonArgument::String(name), PythonArgument::Number(*value)];
                drop(ev);
                let ev = match raw_ev {
                    InputEvent::Raw(ev) => ev,
                };
                run_python_handler(
                    handler.clone(),
                    Some(args),
                    ev,
                    state.transformer.clone(),
                    state.next.values().cloned().collect(),
                )
                .await;
                return;
            }
        }
        _ => {}
    }

    state.next.send_all(raw_ev);
}
