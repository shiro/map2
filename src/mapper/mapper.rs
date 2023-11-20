use std::sync::RwLock;

use crate::*;
use crate::event::InputEvent;
use crate::event_loop::{args_to_py, PythonArgument};
use crate::mapper::{RuntimeAction, RuntimeKeyAction};
use crate::mapper::mapping_functions::*;
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::subscriber::{Subscribable, Subscriber};
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};

#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(KeyActionWithMods, RuntimeAction),
}


fn release_restore_modifiers(
    state: &mut State,
    from_flags: &KeyModifierFlags,
    to_flags: &KeyModifierFlags,
    to_type: &i32,
) -> Vec<evdev_rs::InputEvent> {
    let actual_state = &state.modifiers;
    let mut output_events = vec![];

    // takes into account the actual state of a modifier and decides whether to release/restore it or not
    let mut release_or_restore_modifier = |is_actual_down: &bool, key: &Key| {
        if *to_type == 1 { // restore mods if actual mod is still pressed
            if *is_actual_down {
                output_events.push(
                    KeyAction { key: *key, value: *to_type }.to_input_ev()
                );
            }
        } else { // release mods if actual mod is still pressed (prob. always true since it was necessary to trigger the mapping)
            if *is_actual_down {
                output_events.push(
                    KeyAction { key: *key, value: *to_type }.to_input_ev()
                );
            }
        }
    };

    if from_flags.ctrl && !to_flags.ctrl {
        release_or_restore_modifier(&actual_state.left_ctrl, &*KEY_LEFT_CTRL);
        release_or_restore_modifier(&actual_state.right_ctrl, &*KEY_RIGHT_CTRL);
    }
    if from_flags.shift && !to_flags.shift {
        release_or_restore_modifier(&actual_state.left_shift, &*KEY_LEFT_SHIFT);
        release_or_restore_modifier(&actual_state.right_shift, &*KEY_RIGHT_SHIFT);
    }
    if from_flags.alt && !to_flags.alt {
        release_or_restore_modifier(&actual_state.left_alt, &*KEY_LEFT_ALT);
    }
    if from_flags.right_alt && !to_flags.right_alt {
        release_or_restore_modifier(&actual_state.right_alt, &*KEY_RIGHT_ALT);
    }
    if from_flags.meta && !to_flags.meta {
        release_or_restore_modifier(&actual_state.left_meta, &*KEY_LEFT_META);
        release_or_restore_modifier(&actual_state.right_meta, &*KEY_RIGHT_META);
    }

    if output_events.len() > 0 {
        output_events.push(SYN_REPORT.clone());
    }

    output_events

    // TODO eat keys we just released, un-eat keys we just restored
}

fn run_python_handler(
    handler: &PyObject,
    args: Option<Vec<PythonArgument>>,
) -> Option<String> {
    Python::with_gil(|py| -> Option<String> {
        let asyncio = py.import("asyncio")
            .expect("python runtime error: failed to import 'asyncio', is it installed?");

        let is_async_callback: bool = asyncio
            .call_method1("iscoroutinefunction", (handler.as_ref(py), ))
            .expect("python runtime error: 'iscoroutinefunction' lookup failed")
            .extract()
            .expect("python runtime error: 'iscoroutinefunction' call failed");

        if is_async_callback {
            EVENT_LOOP.lock().unwrap().execute(handler.clone(), args);
            None
        } else {
            let args = args_to_py(py, args.unwrap_or(vec![]));
            let res = handler.call(py, args, None)
                .and_then(|ret| {
                    if ret.is_none(py) { return Ok(None); }
                    Ok(Some(ret.extract::<String>(py)?.clone()))
                });

            match res {
                Ok(ret) => ret,
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
    })
}


#[derive(Default)]
struct Inner {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    state_map: RwLock<HashMap<String, RwLock<State>>>,
    transformer: RwLock<Option<Arc<XKBTransformer>>>,
    mappings: RwLock<Mappings>,

    fallback_handler: RwLock<Option<PyObject>>,
    relative_handler: RwLock<Option<PyObject>>,
    absolute_handler: RwLock<Option<PyObject>>,
}

impl Subscribable for Inner {
    fn handle(&self, id: &str, raw_ev: InputEvent) {
        let mut state_map = self.state_map.read().unwrap();
        let mut state_guard = state_map.get(id);
        if state_guard.is_none() {
            drop(state_map);
            let mut writable_state_map = self.state_map.write().unwrap();
            writable_state_map.insert(id.to_string(), RwLock::new(State::new()));
            drop(writable_state_map);
            state_map = self.state_map.read().unwrap();
            state_guard = state_map.get(id);
        }

        let mappings = self.mappings.read().unwrap();
        let mut state = state_guard.unwrap().write().unwrap();


        let ev = match &raw_ev { InputEvent::Raw(ev) => ev };

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

                // let since_the_epoch = time::SystemTime::now()
                //     .duration_since(time::UNIX_EPOCH)
                //     .unwrap();
                // let mut usec = since_the_epoch.subsec_micros() as i64;

                if let Some(runtime_action) = mappings.get(&from_key_action) {
                    let _subscriber = self.subscriber.load();
                    let subscriber = match _subscriber.deref() {
                        Some(x) => x,
                        None => { return; }
                    };

                    match runtime_action {
                        RuntimeAction::ActionSequence(seq) => {
                            for action in seq {
                                match action {
                                    RuntimeKeyAction::KeyAction(key_action) => {
                                        let ev = key_action.to_input_ev();

                                        subscriber.handle("", InputEvent::Raw(ev));
                                    }
                                    RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                        let mut new_events = release_restore_modifiers(&mut state, from_flags, to_flags, to_type);
                                        // events.append(&mut new_events);
                                        for ev in new_events {
                                            subscriber.handle("", InputEvent::Raw(ev));
                                        }
                                    }
                                }
                            }
                        }
                        RuntimeAction::PythonCallback(from_modifiers, handler) => {
                            // always release all trigger mods before running the callback
                            let mut new_events = release_restore_modifiers(&mut state, from_modifiers, &KeyModifierFlags::new(), &TYPE_UP);
                            for ev in new_events {
                                subscriber.handle("", InputEvent::Raw(ev));
                            }

                            let ret = run_python_handler(&handler, None);
                            if let Some(ret) = ret {
                                let _transformer = self.transformer.read().unwrap();
                                let transformer = _transformer.as_ref().unwrap();

                                let seq = parse_key_sequence_py(&ret, Some(transformer)).unwrap_or_else(|err| {
                                    eprintln!("{err}");
                                    std::process::exit(1);
                                });

                                if let Some(subscriber) = self.subscriber.load().deref() {
                                    for action in seq.to_key_actions() {
                                        subscriber.handle("", InputEvent::Raw(action.to_input_ev()));
                                    }
                                }
                            }
                        }
                        RuntimeAction::NOP => {}
                    }

                    return;
                }

                let was_modifier = event_handlers::update_modifiers(&mut state, &KeyAction::from_input_ev(&ev));

                if let Some(handler) = self.fallback_handler.read().unwrap().as_ref() {
                    // don't trigger the fallback handler with modifier keys
                    if was_modifier { return; }

                    let _transformer = self.transformer.read().unwrap();
                    let transformer = _transformer.as_ref().unwrap();

                    let name = transformer.raw_to_utf(key, &*state.modifiers)
                        .unwrap_or_else(|| format!("{key:?}").to_string());

                    let args = vec![
                        PythonArgument::String(name),
                        PythonArgument::Number(*value),
                    ];

                    let ret = run_python_handler(&handler, Some(args));
                    if let Some(ret) = ret {
                        let seq = parse_key_sequence_py(&ret, Some(transformer)).unwrap_or_else(|err| {
                            eprintln!("{err}");
                            std::process::exit(1);
                        });

                        if let Some(subscriber) = self.subscriber.load().deref() {
                            for action in seq.to_key_actions() {
                                subscriber.handle(id, InputEvent::Raw(action.to_input_ev()));
                            }
                        }
                    }

                    return;
                }
            }
            // rel/abs event
            EvdevInputEvent { event_code, value, .. }
            if matches!(event_code, EventCode::EV_REL(..)) || matches!(event_code, EventCode::EV_ABS(..))
            => {
                let (key, handler) = match event_code {
                    EventCode::EV_REL(key) => (format!("{key:?}").to_string(), self.relative_handler.read().unwrap()),
                    EventCode::EV_ABS(key) => (format!("{key:?}").to_string(), self.absolute_handler.read().unwrap()),
                    _ => unreachable!()
                };
                if let Some(handler) = handler.as_ref() {
                    let _transformer = self.transformer.read().unwrap();
                    let transformer = _transformer.as_ref().unwrap();

                    let name = format!("{key:?}");
                    // remove prefix REL_ / ABS_
                    let name = name[4..name.len()].to_string();
                    let args = vec![
                        PythonArgument::String(name),
                        PythonArgument::Number(*value),
                    ];
                    let ret = run_python_handler(&handler, Some(args));
                    if let Some(ret) = ret {
                        let seq = parse_key_sequence_py(&ret, Some(transformer)).unwrap_or_else(|err| {
                            eprintln!("{err}");
                            std::process::exit(1);
                        });

                        if let Some(subscriber) = self.subscriber.load().deref() {
                            for action in seq.to_key_actions() {
                                subscriber.handle(id, InputEvent::Raw(action.to_input_ev()));
                            }
                        }
                    }
                    return;
                }
            }
            _ => {}
        }

        if let Some(subscriber) = self.subscriber.load().deref() {
            subscriber.handle(id, raw_ev);
        }
    }
}


#[pyclass]
pub struct Mapper {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    inner: Arc<Inner>,
    transformer: Option<Arc<XKBTransformer>>,
    transformer_params: TransformerParams,
}

#[pymethods]
impl Mapper {
    #[new]
    #[pyo3(signature = (* * kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
            None => HashMap::new()
        };

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer_params =
            if kbd_model.is_some() || kbd_layout.is_some() || kbd_variant.is_some() || kbd_options.is_some() {
                TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options)
            } else {
                global::DEFAULT_TRANSFORMER_PARAMS.read().unwrap().clone()
            };

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));

        let inner = Arc::new(Inner {
            subscriber: subscriber.clone(),
            state_map: RwLock::new(HashMap::new()),
            mappings: RwLock::new(Mappings::new()),
            ..Default::default()
        });

        Ok(Self {
            subscriber,
            inner,
            transformer: None,
            transformer_params,
        })
    }

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        self.init_transformer().map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        let from = parse_key_action_with_mods_py(&from, Some(&self.transformer.as_ref().unwrap()))
            .map_err(|err| PyRuntimeError::new_err(
                format!("mapping error on the 'from' side: {}", err.to_string())
            ))?;

        if let Ok(to) = to.extract::<String>(py) {
            let to = parse_key_sequence_py(&to, Some(&self.transformer.as_ref().unwrap()))
                .map_err(|err| PyRuntimeError::new_err(
                    format!("mapping error on the 'to' side: {}", err.to_string())
                ))?;

            self._map_key(from, to)?;
            return Ok(());
        }

        let is_callable = to.as_ref(py).is_callable();

        if is_callable {
            self._map_callback(from, to)?;
            return Ok(());
        }

        Err(PyRuntimeError::new_err("expected a callable object"))
    }

    pub fn map_key(&mut self, from: String, to: String) -> PyResult<()> {
        self.init_transformer().map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        let from = parse_key_action_with_mods_py(&from, Some(&self.transformer.as_ref().unwrap()))
            .map_err(|err| PyRuntimeError::new_err(
                format!("mapping error on the 'from' side: {}", err.to_string())
            ))?;

        let to = parse_key_action_with_mods_py(&to, Some(&self.transformer.as_ref().unwrap()))
            .map_err(|err| PyRuntimeError::new_err(
                format!("mapping error on the 'to' side: {}", err.to_string())
            ))?;

        self._map_key(from, vec![to])?;
        Ok(())
    }

    pub fn map_fallback(&mut self, py: Python, fallback_handler: PyObject) -> PyResult<()> {
        self.init_transformer().map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        if fallback_handler.as_ref(py).is_callable() {
            *self.inner.fallback_handler.write().unwrap() = Some(fallback_handler);
            return Ok(());
        }
        Err(InputError::NotCallable.into())
    }

    pub fn map_relative(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        self.init_transformer().map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        if handler.as_ref(py).is_callable() {
            *self.inner.relative_handler.write().unwrap() = Some(handler);
            return Ok(());
        }
        Err(InputError::NotCallable.into())
    }

    pub fn map_absolute(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        self.init_transformer().map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        if handler.as_ref(py).is_callable() {
            *self.inner.absolute_handler.write().unwrap() = Some(handler);
            return Ok(());
        }
        Err(InputError::NotCallable.into())
    }

    pub fn nop(&mut self, from: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from, Some(&self.transformer.as_ref().unwrap()))
            .map_err(|err| PyRuntimeError::new_err(
                format!("mapping error on the 'from' side: {}", err.to_string())
            ))?;

        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.inner.mappings.write().unwrap().insert(from, RuntimeAction::NOP);
            }
            ParsedKeyAction::KeyClickAction(from) => {
                for value in 0..=2 {
                    let from = KeyActionWithMods::new(from.key, value, from.modifiers);
                    self.inner.mappings.write().unwrap().insert(from, RuntimeAction::NOP);
                }
            }
            ParsedKeyAction::Action(_) => {
                return Err(PyRuntimeError::new_err(format!("this method accepts only button mappings")));
            }
        }

        Ok(())
    }

    pub fn link(&mut self, target: &PyAny) -> PyResult<()> { self._link(target) }

    pub fn snapshot(&self, existing: Option<&KeyMapperSnapshot>) -> PyResult<Option<KeyMapperSnapshot>> {
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
    linkable!();

    pub fn subscribe(&self) -> Subscriber {
        self.inner.clone()
    }

    fn _map_callback(&mut self, from: ParsedKeyAction, to: PyObject) -> PyResult<()> {
        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.inner.mappings.write().unwrap().insert(from, RuntimeAction::PythonCallback(from.modifiers, to));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                self.inner.mappings.write().unwrap().insert(
                    from.to_key_action(1),
                    RuntimeAction::PythonCallback(from.modifiers, to),
                );
                self.inner.mappings.write().unwrap().insert(
                    from.to_key_action(0),
                    RuntimeAction::NOP,
                );
                self.inner.mappings.write().unwrap().insert(
                    from.to_key_action(2),
                    RuntimeAction::NOP,
                );
            }
            ParsedKeyAction::Action(_) => {
                return Err(PyRuntimeError::new_err(format!("this method accepts only button mappings")));
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
                            self.inner.mappings.write().unwrap().insert(mapping.0, mapping.1);
                        }
                        // key action to key action
                        ParsedKeyAction::KeyAction(to) => {
                            let mapping = map_action_to_action(&from, &to);
                            self.inner.mappings.write().unwrap().insert(mapping.0, mapping.1);
                        }
                        // key action to action
                        ParsedKeyAction::Action(to) => {
                            let mapping = map_action_to_action(&from, &to.to_key_action_with_mods(Default::default()));
                            self.inner.mappings.write().unwrap().insert(mapping.0, mapping.1);
                        }
                    }
                    return Ok(());
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                self.inner.mappings.write().unwrap().insert(mapping.0, mapping.1);
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
                return Err(PyRuntimeError::new_err(format!("this method accepts only button mappings")));
            }
        }

        Ok(())
    }

    fn init_transformer(&mut self) -> Result<()> {
        if self.transformer.is_none() {
            let transformer = XKB_TRANSFORMER_REGISTRY.get(&self.transformer_params)?;
            self.transformer = Some(transformer.clone());
            *self.inner.transformer.write().unwrap() = Some(transformer);
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