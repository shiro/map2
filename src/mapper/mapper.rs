use crate::event_loop::{args_to_py, PythonArgument};
use crate::mapper::mapping_functions::*;
use crate::mapper::{RuntimeAction, RuntimeKeyAction};
use crate::python::*;
use crate::subscriber::{SubscribeEvent, Subscriber};
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;

#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(KeyActionWithMods, RuntimeAction),
}

#[derive(Debug, Clone)]
enum PythonReturn {
    String(String),
    Bool(bool),
}

fn run_python_handler(
    handler: &PyObject,
    args: Option<Vec<PythonArgument>>,
    ev: &EvdevInputEvent,
    transformer: &Arc<XKBTransformer>,
    subscriber: Option<&(u64, Subscriber)>,
) {
    let ret = Python::with_gil(|py| -> Result<()> {
        let asyncio = py
            .import("asyncio")
            .expect("python runtime error: failed to import 'asyncio', is it installed?");

        let is_async_callback: bool = asyncio
            .call_method1("iscoroutinefunction", (handler.as_ref(py),))
            .expect("python runtime error: 'iscoroutinefunction' lookup failed")
            .extract()
            .expect("python runtime error: 'iscoroutinefunction' call failed");

        if is_async_callback {
            EVENT_LOOP.lock().unwrap().execute(handler.clone(), args);
            Ok(())
        } else {
            let args = args_to_py(py, args.unwrap_or(vec![]));
            let ret = handler
                .call(py, args, None)
                .map_err(|err| anyhow!("{}", err))
                .and_then(|ret| {
                    if ret.is_none(py) {
                        return Ok(None);
                    }

                    if let Ok(ret) = ret.extract::<String>(py) {
                        return Ok(Some(PythonReturn::String(ret)));
                    }
                    if let Ok(ret) = ret.extract::<bool>(py) {
                        return Ok(Some(PythonReturn::Bool(ret)));
                    }

                    Err(anyhow!("unsupported python return value"))
                })?;

            match ret {
                Some(PythonReturn::String(ret)) => {
                    let seq = parse_key_sequence(&ret, Some(transformer))?;

                    if let Some((path_hash, subscriber)) = subscriber {
                        for action in seq.to_key_actions() {
                            let _ = subscriber
                                .send((*path_hash, InputEvent::Raw(action.to_input_ev())));
                        }
                    }
                }
                Some(PythonReturn::Bool(ret)) if ret => {
                    if let Some((path_hash, subscriber)) = subscriber {
                        let _ = subscriber.send((*path_hash, InputEvent::Raw(ev.clone())));
                    }
                }
                _ => {}
            };
            Ok(())
        }
    });

    if let Err(err) = ret {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[derive(Default)]
struct Inner {
    id: Arc<Uuid>,
    subscriber_map: Arc<RwLock<SubscriberMap>>,
    transformer: Arc<XKBTransformer>,
    mappings: RwLock<Mappings>,

    fallback_handler: RwLock<Option<PyObject>>,
    relative_handler: RwLock<Option<PyObject>>,
    absolute_handler: RwLock<Option<PyObject>>,
}

impl Inner {
    fn handle(&self, path_hash: u64, raw_ev: InputEvent, state: &mut MapperState) {
        let mappings = self.mappings.read().unwrap();

        let subscriber_map = self.subscriber_map.read().unwrap();
        let subscriber = subscriber_map.get(&path_hash);

        let ev = match &raw_ev {
            InputEvent::Raw(ev) => ev,
        };

        match ev {
            // key event
            EvdevInputEvent {
                event_code: EventCode::EV_KEY(key),
                value,
                ..
            } => {
                let mut from_modifiers = KeyModifierFlags::new();
                from_modifiers.ctrl = state.modifiers.is_ctrl();
                from_modifiers.alt = state.modifiers.is_alt();
                from_modifiers.right_alt = state.modifiers.is_right_alt();
                from_modifiers.shift = state.modifiers.is_shift();
                from_modifiers.meta = state.modifiers.is_meta();

                let from_key_action = KeyActionWithMods {
                    key: Key {
                        event_code: ev.event_code,
                    },
                    value: ev.value,
                    modifiers: from_modifiers,
                };

                if let Some(runtime_action) = mappings.get(&from_key_action) {
                    match runtime_action {
                        RuntimeAction::ActionSequence(seq) => {
                            let (path_hash, subscriber) = match subscriber {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            for action in seq {
                                match action {
                                    RuntimeKeyAction::KeyAction(key_action) => {
                                        let ev = key_action.to_input_ev();

                                        let _ = subscriber.send((*path_hash, InputEvent::Raw(ev)));
                                    }
                                    RuntimeKeyAction::ReleaseRestoreModifiers(
                                        from_flags,
                                        to_flags,
                                        to_type,
                                    ) => {
                                        let new_events = release_restore_modifiers(
                                            &state.modifiers,
                                            from_flags,
                                            to_flags,
                                            to_type,
                                        );
                                        // events.append(&mut new_events);
                                        for ev in new_events {
                                            let _ =
                                                subscriber.send((*path_hash, InputEvent::Raw(ev)));
                                        }
                                    }
                                }
                            }
                        }
                        RuntimeAction::PythonCallback(from_modifiers, handler) => {
                            if let Some((path_hash, subscriber)) = subscriber {
                                // always release all trigger mods before running the callback
                                let new_events = release_restore_modifiers(
                                    &state.modifiers,
                                    from_modifiers,
                                    &KeyModifierFlags::new(),
                                    &TYPE_UP,
                                );
                                for ev in new_events {
                                    let _ = subscriber.send((*path_hash, InputEvent::Raw(ev)));
                                }
                            }

                            run_python_handler(&handler, None, ev, &self.transformer, subscriber);
                        }
                        RuntimeAction::NOP => {}
                    }

                    return;
                }

                event_handlers::update_modifiers(
                    &mut state.modifiers,
                    &KeyAction::from_input_ev(&ev),
                );

                if let Some(handler) = self.fallback_handler.read().unwrap().as_ref() {
                    let name = match key {
                        KEY_SPACE => "space".to_string(),
                        KEY_TAB => "tab".to_string(),
                        KEY_ENTER => "enter".to_string(),
                        _ => self
                            .transformer
                            .raw_to_utf(key, &*state.modifiers)
                            .unwrap_or_else(|| {
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

                    run_python_handler(&handler, Some(args), ev, &self.transformer, subscriber);

                    return;
                }
            }
            // rel/abs event
            EvdevInputEvent {
                event_code, value, ..
            } if matches!(event_code, EventCode::EV_REL(..))
                || matches!(event_code, EventCode::EV_ABS(..)) =>
            {
                let (key, handler) = match event_code {
                    EventCode::EV_REL(key) => (
                        format!("{key:?}").to_string(),
                        self.relative_handler.read().unwrap(),
                    ),
                    EventCode::EV_ABS(key) => (
                        format!("{key:?}").to_string(),
                        self.absolute_handler.read().unwrap(),
                    ),
                    _ => unreachable!(),
                };
                if let Some(handler) = handler.as_ref() {
                    let name = format!("{key:?}");
                    // remove prefix REL_ / ABS_
                    let name = name[4..name.len()].to_string();
                    let args = vec![PythonArgument::String(name), PythonArgument::Number(*value)];
                    run_python_handler(&handler, Some(args), ev, &self.transformer, subscriber);
                    return;
                }
            }
            _ => {}
        }

        if let Some((path_hash, subscriber)) = subscriber {
            let _ = subscriber.send((*path_hash, raw_ev));
        }
    }
}

#[pyclass]
pub struct Mapper {
    pub id: Arc<Uuid>,
    subscriber_map: Arc<RwLock<SubscriberMap>>,
    ev_tx: Subscriber,
    inner: Arc<Inner>,
    transformer: Arc<XKBTransformer>,
}

#[pymethods]
impl Mapper {
    #[new]
    #[pyo3(signature = (* * kwargs))]
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
            .get(&TransformerParams::new(
                kbd_model,
                kbd_layout,
                kbd_variant,
                kbd_options,
            ))
            .map_err(err_to_py)?;

        let subscriber_map: Arc<RwLock<SubscriberMap>> = Arc::new(RwLock::new(HashMap::new()));
        let id = Arc::new(Uuid::new_v4());

        let inner = Arc::new(Inner {
            id: id.clone(),
            subscriber_map: subscriber_map.clone(),
            mappings: RwLock::new(Mappings::new()),
            ..Default::default()
        });

        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::unbounded_channel::<SubscribeEvent>();

        let _inner = inner.clone();
        get_runtime().spawn(async move {
            let mut state_map = HashMap::new();

            loop {
                let (path_hash, ev) = ev_rx.recv().await.unwrap();
                let state = state_map.entry(path_hash).or_default();
                _inner.handle(path_hash, ev, state);
            }
        });

        Ok(Self {
            id,
            subscriber_map,
            ev_tx,
            inner,
            transformer,
        })
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
            *self.inner.fallback_handler.write().unwrap() = Some(fallback_handler);
            return Ok(());
        }
        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_relative(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        if handler.as_ref(py).is_callable() {
            *self.inner.relative_handler.write().unwrap() = Some(handler);
            return Ok(());
        }
        Err(ApplicationError::NotCallable.into())
    }

    pub fn map_absolute(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        if handler.as_ref(py).is_callable() {
            *self.inner.absolute_handler.write().unwrap() = Some(handler);
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
                self.inner
                    .mappings
                    .write()
                    .unwrap()
                    .insert(from, RuntimeAction::NOP);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                for value in 0..=2 {
                    let from = KeyActionWithMods::new(from.key, value, from.modifiers);
                    self.inner
                        .mappings
                        .write()
                        .unwrap()
                        .insert(from, RuntimeAction::NOP);
                }
            }
            ParsedKeyAction::Action(_) => {
                return Err(ApplicationError::NonButton.into());
            }
        }

        Ok(())
    }

    pub fn snapshot(
        &self,
        existing: Option<&KeyMapperSnapshot>,
    ) -> PyResult<Option<KeyMapperSnapshot>> {
        if let Some(existing) = existing {
            *self.inner.mappings.write().unwrap() = existing.mappings.clone();
            *self.inner.fallback_handler.write().unwrap() = existing.fallback_handler.clone();
            *self.inner.relative_handler.write().unwrap() = existing.relative_handler.clone();
            *self.inner.absolute_handler.write().unwrap() = existing.absolute_handler.clone();
            return Ok(None);
        }

        Ok(Some(KeyMapperSnapshot {
            mappings: self.inner.mappings.read().unwrap().clone(),
            fallback_handler: self.inner.fallback_handler.read().unwrap().clone(),
            relative_handler: self.inner.relative_handler.read().unwrap().clone(),
            absolute_handler: self.inner.absolute_handler.read().unwrap().clone(),
        }))
    }
}

impl Mapper {
    pub fn link(&mut self, mut path: Vec<Arc<Uuid>>, target: &PyAny) -> PyResult<()> {
        use crate::subscriber::*;

        let target = match add_event_subscription(target) {
            Some(target) => target,
            None => {
                return Err(ApplicationError::InvalidLinkTarget.into());
            }
        };

        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        let path_hash = h.finish();

        let mut h = DefaultHasher::new();
        path.push(self.id.clone());
        path.hash(&mut h);
        let next_path_hash = h.finish();

        self.subscriber_map
            .write()
            .unwrap()
            .insert(path_hash, (next_path_hash, target));
        Ok(())
    }

    pub fn subscribe(&self) -> Subscriber {
        self.ev_tx.clone()
    }

    fn _map_callback(&mut self, from: ParsedKeyAction, to: PyObject) -> PyResult<()> {
        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.inner
                    .mappings
                    .write()
                    .unwrap()
                    .insert(from, RuntimeAction::PythonCallback(from.modifiers, to));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                self.inner.mappings.write().unwrap().insert(
                    from.to_key_action(1),
                    RuntimeAction::PythonCallback(from.modifiers, to),
                );
                self.inner
                    .mappings
                    .write()
                    .unwrap()
                    .insert(from.to_key_action(0), RuntimeAction::NOP);
                self.inner
                    .mappings
                    .write()
                    .unwrap()
                    .insert(from.to_key_action(2), RuntimeAction::NOP);
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
                            self.inner
                                .mappings
                                .write()
                                .unwrap()
                                .insert(mapping.0, mapping.1);
                        }
                        // key action to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mapping = map_action_to_action(&from, &to);
                            self.inner
                                .mappings
                                .write()
                                .unwrap()
                                .insert(mapping.0, mapping.1);
                        }
                        // key action to action
                        ParsedKeyAction::Action(to) => {
                            let mapping = map_action_to_action(
                                &from,
                                &to.to_key_action_with_mods(Default::default()),
                            );
                            self.inner
                                .mappings
                                .write()
                                .unwrap()
                                .insert(mapping.0, mapping.1);
                        }
                    }
                    return Ok(());
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                self.inner
                    .mappings
                    .write()
                    .unwrap()
                    .insert(mapping.0, mapping.1);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    match to.remove(0) {
                        // click to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mappings = map_click_to_click(&from, &to);

                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                self.inner.mappings.write().unwrap().insert(from, to);
                            });
                        }
                        // click to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mappings = map_click_to_action(&from, &to);
                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                self.inner.mappings.write().unwrap().insert(from, to);
                            });
                        }
                        // click to action
                        ParsedKeyAction::Action(to) => {
                            let to = to.to_key_action_with_mods(Default::default());
                            let mappings = map_click_to_action(&from, &to);
                            IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                                self.inner.mappings.write().unwrap().insert(from, to);
                            });
                        }
                    };
                    return Ok(());
                }

                // click to seq
                let mappings = map_click_to_seq(from, to);
                IntoIterator::into_iter(mappings).for_each(|(from, to)| {
                    self.inner.mappings.write().unwrap().insert(from, to);
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
