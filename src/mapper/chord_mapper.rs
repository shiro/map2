use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeKeyAction;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use nom::Slice;

use self::subscriber::SubscriberNew;

type Mappings = HashMap<Vec<Key>, RuntimeAction>;

struct State {
    pub modifiers: Arc<KeyModifierState>,
    stack: Vec<Key>,
    ignored_keys: HashSet<Key>,
    pressed_keys: HashSet<Key>,
    interval: Option<tokio::task::JoinHandle<()>>,
    msg_tx: tokio::sync::mpsc::UnboundedSender<Msg>,
}

enum Msg {
    Callback(InputEvent),
}

struct SharedState {
    transformer: Arc<XKBTransformer>,
    mappings: Mappings,
    chorded_keys: HashSet<Key>,
}

impl State {
    fn handle(
        &mut self,
        raw_ev: InputEvent,
        next: Option<&SubscriberNew>,
        shared_state: &SharedState,
    ) {
        let ev = match &raw_ev {
            InputEvent::Raw(ev) => ev,
        };

        let _key = Key {
            event_code: ev.event_code,
        };

        match ev {
            EvdevInputEvent {
                event_code: EventCode::EV_KEY(key),
                value,
                ..
            } => {
                let mut from_modifiers = KeyModifierFlags::new();
                from_modifiers.ctrl = self.modifiers.is_ctrl();
                from_modifiers.alt = self.modifiers.is_alt();
                from_modifiers.right_alt = self.modifiers.is_right_alt();
                from_modifiers.shift = self.modifiers.is_shift();
                from_modifiers.meta = self.modifiers.is_meta();

                event_handlers::update_modifiers(
                    &mut self.modifiers,
                    &KeyAction::from_input_ev(&ev),
                );

                match ev.value {
                    TYPE_DOWN => {
                        let should_handle = shared_state.chorded_keys.contains(&_key)
                            && self.pressed_keys.iter().all(|x| self.stack.contains(x));

                        self.pressed_keys.insert(_key.clone());

                        if should_handle {
                            self.stack.push(_key.clone());

                            if self.stack.len() == 2 {
                                self.handle_cb(raw_ev, next, shared_state);
                            } else {
                                let msg_tx = self.msg_tx.clone();
                                self.interval = Some(tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(50))
                                        .await;
                                    msg_tx.send(Msg::Callback(raw_ev));
                                }));
                            }
                        } else {
                            let next = match next {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            if let Some(task) = self.interval.take() {
                                task.abort();
                            };

                            for k in self.stack.iter() {
                                let _ = next.send(InputEvent::Raw(k.to_input_ev(TYPE_DOWN)));
                                let _ = next.send(InputEvent::Raw(k.to_input_ev(TYPE_UP)));
                                self.ignored_keys.insert(k.clone());
                            }
                            self.stack.clear();

                            let _ = next.send(raw_ev);
                        }
                    }
                    TYPE_UP => {
                        self.pressed_keys.remove(&_key);

                        let next = match next {
                            Some(x) => x,
                            None => {
                                return;
                            }
                        };

                        if let Some(task) = self.interval.take() {
                            task.abort();
                        };

                        if let Some(pos) = self.stack.iter().position(|x| x == &_key) {
                            self.stack.remove(pos);

                            if !self.ignored_keys.remove(&_key) {
                                let _ = next.send(InputEvent::Raw(_key.to_input_ev(TYPE_DOWN)));
                                let _ = next.send(raw_ev);
                            }
                        } else {
                            for k in self.stack.iter() {
                                let _ = next.send(InputEvent::Raw(k.to_input_ev(TYPE_DOWN)));
                                let _ = next.send(InputEvent::Raw(k.to_input_ev(TYPE_UP)));
                            }

                            self.stack.clear();

                            if !self.ignored_keys.remove(&_key) {
                                let _ = next.send(raw_ev);
                            }
                        }
                    }
                    TYPE_REPEAT => {
                        if self.ignored_keys.contains(&_key) {
                            return;
                        }
                        if self.stack.is_empty() {
                            // let subscriber_map = self.subscriber_map.read().unwrap();
                            // let subscriber = subscriber_map.get(&path_hash);
                            let next = match next {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            let _ = next.send(raw_ev);
                        };
                    }
                    _ => unreachable!(),
                };
            }
            _ => {}
        }
    }

    // fired after the chord timeout has passed, submits the keys held on the stack
    fn handle_cb(
        &mut self,
        raw_ev: InputEvent,
        next: Option<&SubscriberNew>,
        shared_state: &SharedState,
    ) {
        // let mut inner = _inner.write().unwrap();
        let ev = match &raw_ev {
            InputEvent::Raw(ev) => ev,
        };

        if let Some(task) = self.interval.take() {
            task.abort();
        };

        if let Some(action) = shared_state.mappings.get(&self.stack) {
            for k in self.stack.iter().cloned() {
                self.ignored_keys.insert(k);
            }

            // for action in output {
            match action {
                RuntimeAction::ActionSequence(seq) => {
                    for action in seq {
                        match action {
                            RuntimeKeyAction::KeyAction(key_action) => {
                                if let Some(next) = next {
                                    let ev = key_action.to_input_ev();
                                    let _ = next.send(InputEvent::Raw(ev));
                                }
                            }
                            RuntimeKeyAction::ReleaseRestoreModifiers(
                                from_flags,
                                to_flags,
                                to_type,
                            ) => {
                                let new_events = release_restore_modifiers(
                                    &self.modifiers,
                                    &from_flags,
                                    &to_flags,
                                    &to_type,
                                );
                                if let Some(next) = next {
                                    for ev in new_events {
                                        let _ = next.send(InputEvent::Raw(ev));
                                    }
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
            // }
        } else {
            let next = match next {
                Some(x) => x,
                None => {
                    return;
                }
            };

            // only one key on the stack
            if self.stack.len() == 1 && self.stack[0].event_code == ev.event_code {
                let _ = next.send(InputEvent::Raw(ev.clone()));
            } else {
                // no match, send all buffered keys from stack
                for k in self.stack.iter() {
                    let _ = next.send(InputEvent::Raw(k.to_input_ev(1)));
                    let _ = next.send(InputEvent::Raw(k.to_input_ev(0)));
                }
            }
            self.stack.clear();
        }
    }
}

#[pyclass]
pub struct ChordMapper {
    pub id: Uuid,
    shared_state: Arc<RwLock<SharedState>>,
    transformer: Arc<XKBTransformer>,
    tmp_next: Mutex<Option<SubscriberNew>>,
}

#[pymethods]
impl ChordMapper {
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

        let id = Uuid::new_v4();

        let shared_state = Arc::new(RwLock::new(SharedState {
            transformer: transformer.clone(),
            chorded_keys: Default::default(),
            mappings: Default::default(),
        }));

        Ok(Self {
            id,
            shared_state,
            transformer,
            tmp_next: Default::default(),
        })
    }

    pub fn map(&mut self, py: Python, from: Vec<String>, to: PyObject) -> PyResult<()> {
        // if from.len() > 32 {
        //     return Err(PyRuntimeError::new_err(
        //         "'from' side cannot be longer than 32 character",
        //     ));
        // }

        let mut from_parsed = from
            .into_iter()
            .map(|x| parse_key(&x, Some(&self.transformer)))
            .collect::<Result<Vec<_>>>()
            .map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'from' side:\n{}",
                    ApplicationError::KeyParse(err.to_string()),
                ))
            })?;

        // let from_seq: Vec<KeyClickActionWithMods> =
        //     parse_key_sequence(&from, Some(&self.transformer))
        //         .map_err(|err| {
        //             PyRuntimeError::new_err(format!(
        //                 "mapping error on the 'from' side:\n{}",
        //                 ApplicationError::KeyParse(err.to_string()),
        //             ))
        //         })?
        //         .into_iter()
        //         .map(|x| match x {
        //             ParsedKeyAction::KeyClickAction(x) => Some(x),
        //             _ => None,
        //         })
        //         .collect::<Option<Vec<_>>>()
        //         .ok_or_else(|| PyRuntimeError::new_err("invalid key sequence"))?;
        //

        let mut shared_state = self.shared_state.write().unwrap();

        // TODO fix code duplication
        if to.as_ref(py).is_callable() {
            // self._map_callback(from, to)?;
            let to = RuntimeAction::PythonCallback(Default::default(), to);

            // mark chorded keys
            shared_state
                .chorded_keys
                .extend(from_parsed.iter().cloned());

            // insert all combinations
            shared_state
                .mappings
                .insert(from_parsed.clone(), to.clone());
            from_parsed.reverse();
            shared_state.mappings.insert(from_parsed, to);
            return Ok(());
        }

        let to = to.extract::<String>(py).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'to' side:\n{}",
                ApplicationError::InvalidInputType {
                    type_: "String".to_string(),
                }
            ))
        })?;
        let to_parsed = parse_key_sequence(&to, Some(&self.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'to' side:\n{}",
                ApplicationError::KeySequenceParse(err.to_string()),
            ))
        })?;

        let mut to: Vec<RuntimeKeyAction> = to_parsed
            .to_key_actions()
            .into_iter()
            .map(|action| RuntimeKeyAction::KeyAction(action))
            .collect();

        // mark chorded keys
        shared_state
            .chorded_keys
            .extend(from_parsed.iter().cloned());

        let to = RuntimeAction::ActionSequence(to);

        // insert all combinations
        shared_state
            .mappings
            .insert(from_parsed.clone(), to.clone());
        from_parsed.reverse();
        shared_state.mappings.insert(from_parsed, to);

        Ok(())
    }

    pub fn snapshot(
        &self,
        existing: Option<&ChordMapperSnapshot>,
    ) -> PyResult<Option<ChordMapperSnapshot>> {
        if let Some(existing) = existing {
            let mut shared_state = self.shared_state.write().unwrap();
            shared_state.mappings = existing.mappings.clone();
            shared_state.chorded_keys =
                shared_state
                    .mappings
                    .keys()
                    .fold(HashSet::new(), |mut acc, e| {
                        acc.extend(e.iter().cloned());
                        acc
                    });
            return Ok(None);
        }

        let mut shared_state = self.shared_state.read().unwrap();

        Ok(Some(ChordMapperSnapshot {
            mappings: shared_state.mappings.clone(),
        }))
    }
}

impl ChordMapper {
    pub fn link(&mut self, target: Option<SubscriberNew>) -> PyResult<()> {
        use crate::subscriber::*;

        match target {
            Some(target) => {
                *self.tmp_next.lock().unwrap() = Some(target);
            }
            None => {}
        };
        Ok(())
    }

    pub fn subscribe(&self) -> SubscriberNew {
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::unbounded_channel::<InputEvent>();
        let next = self.tmp_next.lock().unwrap().take();

        let _shared_state = self.shared_state.clone();
        get_runtime().spawn(async move {
            let (msg_tx, mut msg_rx) = tokio::sync::mpsc::unbounded_channel();
            let mut state = State {
                modifiers: Default::default(),
                stack: Default::default(),
                ignored_keys: Default::default(),
                pressed_keys: Default::default(),
                interval: Default::default(),
                msg_tx,
            };
            loop {
                tokio::select! {
                    Some(ev) = ev_rx.recv() => {
                        let shared_state = _shared_state.read().unwrap();
                        state.handle(ev, next.as_ref(), &shared_state);
                    }
                    msg = msg_rx.recv() => {
                        let ev = match msg.unwrap() {
                            Msg::Callback(ev) => ev,
                        };
                        let shared_state = _shared_state.read().unwrap();
                        state.handle_cb(ev, next.as_ref(), &shared_state);
                    }
                };
            }
        });

        ev_tx.clone()
    }
}

#[pyclass]
pub struct ChordMapperSnapshot {
    mappings: Mappings,
}
