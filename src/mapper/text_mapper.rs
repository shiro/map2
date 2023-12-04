use crate::*;
use crate::mapper::RuntimeKeyAction;
use crate::mapper::mapping_functions::*;
use crate::python::*;
use crate::subscriber::{SubscribeEvent, Subscriber};
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};

type Mappings = HashMap<String, Vec<RuntimeKeyAction>>;


#[derive(Default)]
struct State {
    pub modifiers: Arc<KeyModifierState>,
    pub window: Vec<char>,
}


impl State {
    pub fn new() -> Self {
        State {
            modifiers: Arc::new(KeyModifierState::new()),
            window: vec![],
        }
    }
}

fn _map(from: &KeyClickActionWithMods, to: Vec<ParsedKeyAction>) -> Vec<RuntimeKeyAction> {
    let mut seq: Vec<RuntimeKeyAction> = to.to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();
    seq.insert(0, RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), KeyModifierFlags::new(), TYPE_UP));
    seq
}


#[derive(Default)]
struct Inner {
    id: Arc<Uuid>,
    subscriber_map: Arc<RwLock<SubscriberMap>>,
    transformer: Arc<XKBTransformer>,
    mappings: RwLock<Mappings>,
}

impl Inner {
    fn handle(&self, path_hash: u64, raw_ev: InputEvent, state: &mut State) {
        let mappings = self.mappings.read().unwrap();

        let subscriber_map = self.subscriber_map.read().unwrap();
        let subscriber = subscriber_map.get(&path_hash);

        let ev = match &raw_ev { InputEvent::Raw(ev) => ev };

        match ev {
            EvdevInputEvent { event_code: EventCode::EV_KEY(KEY_BACKSPACE), value: 1, .. } => {
                state.window.pop();
            }
            EvdevInputEvent { event_code: EventCode::EV_KEY(KEY_DELETE), value: 1, .. } => {
                state.window.remove(0);
            }
            // key event
            EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
                let mut from_modifiers = KeyModifierFlags::new();
                from_modifiers.ctrl = state.modifiers.is_ctrl();
                from_modifiers.alt = state.modifiers.is_alt();
                from_modifiers.right_alt = state.modifiers.is_right_alt();
                from_modifiers.shift = state.modifiers.is_shift();
                from_modifiers.meta = state.modifiers.is_meta();

                if ev.value == 1 {
                    let key = self.transformer.raw_to_utf(&key, &state.modifiers);

                    if let Some(key) = key {
                        state.window.push(key.chars().next().unwrap());
                        if state.window.len() > 5 {
                            state.window.remove(0);
                        }

                        let mut hit = None;

                        for i in (0..state.window.len()).rev() {
                            let search: String = state.window.iter().skip(i).collect();
                            if let Some(x) = mappings.get(&search) {
                                hit = Some((x, search.len()));
                            }
                        }

                        if let Some((to, from_len)) = hit {
                            state.window.clear();
                            let (path_hash, subscriber) = match subscriber {
                                Some(x) => x,
                                None => { return; }
                            };

                            for _ in 1..from_len {
                                let _ = subscriber.send((*path_hash, InputEvent::Raw(
                                    Key::from(KEY_BACKSPACE).to_input_ev(1),
                                )));
                                let _ = subscriber.send((*path_hash, InputEvent::Raw(
                                    Key::from(KEY_BACKSPACE).to_input_ev(0),
                                )));
                            }

                            for action in to {
                                match action {
                                    RuntimeKeyAction::KeyAction(key_action) => {
                                        let ev = key_action.to_input_ev();
                                        let _ = subscriber.send((*path_hash, InputEvent::Raw(ev)));
                                    }
                                    RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                        let new_events = release_restore_modifiers(
                                            &state.modifiers, from_flags, to_flags, to_type,
                                        );
                                        // events.append(&mut new_events);
                                        for ev in new_events {
                                            let _ = subscriber.send((*path_hash, InputEvent::Raw(ev)));
                                        }
                                    }
                                }
                            }
                            return;
                        }
                    }
                }

                event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));
            }
            _ => {}
        }

        if let Some((path_hash, subscriber)) = subscriber {
            let _ = subscriber.send((*path_hash, raw_ev));
        }
    }
}


#[pyclass]
pub struct TextMapper {
    pub id: Arc<Uuid>,
    subscriber_map: Arc<RwLock<SubscriberMap>>,
    ev_tx: Subscriber,
    inner: Arc<Inner>,
    transformer: Arc<XKBTransformer>,
}

#[pymethods]
impl TextMapper {
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
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
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


    pub fn map(&mut self, from: String, to: String) -> PyResult<()> {
        let from_seq: Vec<KeyClickActionWithMods> = parse_key_sequence(&from, Some(&self.transformer))
            .map_err(|err| PyRuntimeError::new_err(format!(
                "mapping error on the 'from' side:\n{}",
                ApplicationError::KeyParse(err.to_string()),
            )))?
            .into_iter()
            .map(|x| {
                match x {
                    ParsedKeyAction::KeyClickAction(x) => Some(x),
                    _ => None,
                }
            })
            .collect::<Option<Vec<_>>>()
            .unwrap();

        let to = parse_key_sequence(&to, Some(&self.transformer))
            .map_err(|err| PyRuntimeError::new_err(format!(
                "mapping error on the 'to' side:\n{}",
                ApplicationError::KeySequenceParse(err.to_string()),
            )))?;

        let to = _map(from_seq.last().unwrap(), to);

        self.inner.mappings.write().unwrap().insert(from, to);

        Ok(())
    }

    pub fn snapshot(&self, existing: Option<&TextMapperSnapshot>) -> PyResult<Option<TextMapperSnapshot>> {
        if let Some(existing) = existing {
            *self.inner.mappings.write().unwrap() = existing.mappings.clone();
            return Ok(None);
        }

        Ok(Some(TextMapperSnapshot {
            mappings: self.inner.mappings.read().unwrap().clone(),
        }))
    }
}

impl TextMapper {
    pub fn link(&mut self, mut path: Vec<Arc<Uuid>>, target: &PyAny) -> PyResult<()> {
        use crate::subscriber::*;

        let target = match add_event_subscription(target) {
            Some(target) => target,
            None => { return Err(ApplicationError::InvalidLinkTarget.into()); }
        };

        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        let path_hash = h.finish();

        let mut h = DefaultHasher::new();
        path.push(self.id.clone());
        path.hash(&mut h);
        let next_path_hash = h.finish();

        self.subscriber_map.write().unwrap().insert(path_hash, (next_path_hash, target));
        Ok(())
    }

    pub fn subscribe(&self) -> Subscriber {
        self.ev_tx.clone()
    }
}

#[pyclass]
pub struct TextMapperSnapshot {
    mappings: Mappings,
}
