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

const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Default)]
struct State {
    name: String,
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
        let state = Arc::new(Mutex::new(State { transformer, ..Default::default() }));
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

    pub fn map_fallback(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.bind(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.fallback_handler = Some(Arc::new(handler));
        Ok(())
    }

    pub fn map_relative(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.bind(py).is_callable() {
            return Err(ApplicationError::NotCallable.into());
        }
        state.relative_handler = Some(Arc::new(handler));
        Ok(())
    }

    pub fn map_absolute(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        let mut state = self.state.blocking_lock();
        if !handler.bind(py).is_callable() {
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

    // pub fn replace(&self, other: &PyBound<PyAny>) {
    //     let state = self.state.blocking_lock();
    //     state.prev.values()
    // }

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

                        let handler = handler.clone();
                        let transformer = state.transformer.clone();
                        let next = state.next.values().cloned().collect();
                        drop(state);
                        run_python_handler(handler, None, ev, transformer, next).await;
                    }
                    RuntimeAction::NOP => {}
                }

                return;
            }

            event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));

            if let Some(handler) = state.fallback_handler.as_ref() {
                let args =
                    Some(python_callback_args(&EventCode::EV_KEY(*key), &state.modifiers, *value, &state.transformer));
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
        // rel/abs event
        EvdevInputEvent { event_code: ev, value, .. }
            if matches!(ev, EventCode::EV_REL(..)) || matches!(ev, EventCode::EV_ABS(..)) =>
        {
            let handler = match ev {
                EventCode::EV_REL(key) => &state.relative_handler,
                EventCode::EV_ABS(key) => &state.absolute_handler,
                _ => unreachable!(),
            };
            if let Some(handler) = handler.as_ref() {
                let args = Some(python_callback_args(ev, &state.modifiers, *value, &state.transformer));
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
