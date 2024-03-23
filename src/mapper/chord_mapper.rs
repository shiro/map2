use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeKeyAction;
use crate::python::*;
use crate::subscriber::{SubscribeEvent, Subscriber};
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use nom::Slice;

type Mappings = HashMap<Vec<Key>, Vec<RuntimeKeyAction>>;

#[derive(Default)]
struct State {
    pub modifiers: Arc<KeyModifierState>,
    stack: Vec<Key>,
    ignored_keys: HashSet<Key>,
    pressed_keys: HashSet<Key>,
    interval: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Default)]
struct SharedState {
    mappings: Mappings,
    chorded_keys: HashSet<Key>,
}

enum Msg {
    Callback(u64, InputEvent),
}

struct Inner {
    id: Arc<Uuid>,
    transformer: Arc<XKBTransformer>,
    subscriber_map: Arc<RwLock<SubscriberMap>>,
    msg_tx: tokio::sync::mpsc::UnboundedSender<Msg>,
    shared_state: RwLock<SharedState>,
}

impl Inner {
    fn new(id: Arc<Uuid>, msg_tx: tokio::sync::mpsc::UnboundedSender<Msg>) -> Self {
        Self {
            id,
            transformer: Default::default(),
            subscriber_map: Default::default(),
            msg_tx,
            shared_state: Default::default(),
        }
    }
}

impl Inner {
    fn handle(&self, path_hash: u64, raw_ev: InputEvent, state: &mut State) {
        let shared = self.shared_state.read().unwrap();
        let mappings = &shared.mappings;

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
                from_modifiers.ctrl = state.modifiers.is_ctrl();
                from_modifiers.alt = state.modifiers.is_alt();
                from_modifiers.right_alt = state.modifiers.is_right_alt();
                from_modifiers.shift = state.modifiers.is_shift();
                from_modifiers.meta = state.modifiers.is_meta();

                event_handlers::update_modifiers(
                    &mut state.modifiers,
                    &KeyAction::from_input_ev(&ev),
                );

                match ev.value {
                    TYPE_DOWN => {
                        let should_handle = shared.chorded_keys.contains(&_key)
                            && state.pressed_keys.iter().all(|x| state.stack.contains(x));

                        state.pressed_keys.insert(_key.clone());

                        if should_handle {
                            state.stack.push(_key.clone());

                            if state.stack.len() == 2 {
                                self.handle_cb(path_hash, raw_ev, state);
                            } else {
                                let msg_tx = self.msg_tx.clone();

                                let task = tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(50))
                                        .await;
                                    msg_tx.send(Msg::Callback(path_hash, raw_ev));
                                });
                            }
                        } else {
                            let subscriber_map = self.subscriber_map.read().unwrap();
                            let subscriber = subscriber_map.get(&path_hash);
                            let (path_hash, subscriber) = match subscriber {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            if let Some(task) = state.interval.take() {
                                task.abort();
                            };

                            for k in state.stack.iter() {
                                let _ = subscriber
                                    .send((*path_hash, InputEvent::Raw(k.to_input_ev(TYPE_DOWN))));
                                let _ = subscriber
                                    .send((*path_hash, InputEvent::Raw(k.to_input_ev(TYPE_UP))));
                                state.ignored_keys.insert(k.clone());
                            }
                            state.stack.clear();

                            let _ = subscriber.send((*path_hash, raw_ev));
                        }
                    }
                    TYPE_UP => {
                        state.pressed_keys.remove(&_key);

                        let subscriber_map = self.subscriber_map.read().unwrap();
                        let subscriber = subscriber_map.get(&path_hash);
                        let (path_hash, subscriber) = match subscriber {
                            Some(x) => x,
                            None => {
                                return;
                            }
                        };

                        if let Some(task) = state.interval.take() {
                            task.abort();
                        };

                        for k in state.stack.iter() {
                            let _ = subscriber
                                .send((*path_hash, InputEvent::Raw(k.to_input_ev(TYPE_DOWN))));
                            let _ = subscriber
                                .send((*path_hash, InputEvent::Raw(k.to_input_ev(TYPE_UP))));
                        }

                        let should_handle =
                            !state.ignored_keys.remove(&_key) && !state.stack.contains(&_key);
                        state.stack.clear();

                        if should_handle {
                            let _ = subscriber.send((*path_hash, raw_ev));
                        }
                    }
                    TYPE_REPEAT => {
                        if state.ignored_keys.contains(&_key) {
                            return;
                        }
                        if state.stack.is_empty() {
                            // if let Some(task) = state.interval.take() {
                            //     task.abort();
                            // };

                            let subscriber_map = self.subscriber_map.read().unwrap();
                            let subscriber = subscriber_map.get(&path_hash);
                            let (path_hash, subscriber) = match subscriber {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            // for k in state.stack.iter() {
                            //     let _ = subscriber
                            //         .send((*path_hash, InputEvent::Raw(k.to_input_ev(TYPE_DOWN))));
                            //     let _ = subscriber
                            //         .send((*path_hash, InputEvent::Raw(k.to_input_ev(TYPE_UP))));
                            // }
                            // state.stack.clear();

                            let _ = subscriber.send((*path_hash, raw_ev));
                        };
                    }
                    _ => unreachable!(),
                };
            }
            _ => {}
        }
    }

    // fired after the chord timeout has passed, submits the keys held on the stack
    fn handle_cb(&self, path_hash: u64, raw_ev: InputEvent, state: &mut State) {
        // let mut inner = _inner.write().unwrap();
        let ev = match &raw_ev {
            InputEvent::Raw(ev) => ev,
        };

        let shared = self.shared_state.read().unwrap();
        let mappings = &shared.mappings;

        if let Some(task) = state.interval.take() {
            task.abort();
        };

        let subscriber_map = self.subscriber_map.read().unwrap();
        let subscriber = subscriber_map.get(&path_hash);
        let (path_hash, subscriber) = match subscriber {
            Some(x) => x,
            None => {
                return;
            }
        };

        if let Some(output) = mappings.get(&state.stack) {
            for k in state.stack.drain(0..state.stack.len()) {
                state.ignored_keys.insert(k);
            }

            for action in output {
                match action {
                    RuntimeKeyAction::KeyAction(key_action) => {
                        let ev = key_action.to_input_ev();
                        let _ = subscriber.send((*path_hash, InputEvent::Raw(ev)));
                    }
                    RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                        let new_events = release_restore_modifiers(
                            &state.modifiers,
                            &from_flags,
                            &to_flags,
                            &to_type,
                        );
                        for ev in new_events {
                            let _ = subscriber.send((*path_hash, InputEvent::Raw(ev)));
                        }
                    }
                }
            }
        } else {
            // only one key on the stack
            if state.stack.len() == 1 && state.stack[0].event_code == ev.event_code {
                let _ = subscriber.send((*path_hash, InputEvent::Raw(ev.clone())));
            } else {
                // no match, send all buffered keys from stack
                for k in state.stack.iter() {
                    let _ = subscriber.send((*path_hash, InputEvent::Raw(k.to_input_ev(1))));
                    let _ = subscriber.send((*path_hash, InputEvent::Raw(k.to_input_ev(0))));
                }
            }
        }

        state.stack.clear();
    }
}

#[pyclass]
pub struct ChordMapper {
    pub id: Arc<Uuid>,
    subscriber_map: Arc<RwLock<SubscriberMap>>,
    ev_tx: Subscriber,
    inner: Arc<Inner>,
    transformer: Arc<XKBTransformer>,
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

        let id = Arc::new(Uuid::new_v4());

        let (msg_tx, mut msg_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::unbounded_channel::<SubscribeEvent>();

        let inner = Arc::new(Inner::new(id.clone(), msg_tx));
        let subscriber_map = inner.subscriber_map.clone();

        let _inner = inner.clone();
        get_runtime().spawn(async move {
            let mut state_map = HashMap::new();

            loop {
                tokio::select! {
                    Some(res) = ev_rx.recv() => {
                        let (path_hash, ev) = res;
                        let state = state_map.entry(path_hash).or_default();
                        _inner.handle(path_hash, ev, state);
                    }
                    ev = msg_rx.recv() => {
                        let (path_hash, ev) = match ev.unwrap() {
                            Msg::Callback(path_hash, ev) => (path_hash, ev),
                        };
                        let state = state_map.entry(path_hash).or_default();
                        _inner.handle_cb(path_hash, ev, state);
                    }
                };
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

    pub fn map(&mut self, from: Vec<String>, to: String) -> PyResult<()> {
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

        let mut shared = self.inner.shared_state.write().unwrap();

        // mark chorded keys
        shared.chorded_keys.extend(from_parsed.iter().cloned());

        // insert all combinations
        shared.mappings.insert(from_parsed.clone(), to.clone());
        from_parsed.reverse();
        shared.mappings.insert(from_parsed, to);

        Ok(())
    }

    pub fn snapshot(
        &self,
        existing: Option<&ChordMapperSnapshot>,
    ) -> PyResult<Option<ChordMapperSnapshot>> {
        if let Some(existing) = existing {
            let mut shared = self.inner.shared_state.write().unwrap();
            shared.mappings = existing.mappings.clone();
            shared.chorded_keys = shared.mappings.keys().fold(HashSet::new(), |mut acc, e| {
                acc.extend(e.iter().cloned());
                acc
            });
            return Ok(None);
        }

        let mut shared = self.inner.shared_state.read().unwrap();

        Ok(Some(ChordMapperSnapshot {
            mappings: shared.mappings.clone(),
        }))
    }
}

impl ChordMapper {
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
}

#[pyclass]
pub struct ChordMapperSnapshot {
    mappings: Mappings,
}
