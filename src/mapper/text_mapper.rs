use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeKeyAction;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use nom::Slice;

use super::suffix_tree::SuffixTree;

type Mappings = SuffixTree<Vec<RuntimeKeyAction>>;

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
    let mut seq: Vec<RuntimeKeyAction> = to
        .to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();
    seq.insert(
        0,
        RuntimeKeyAction::ReleaseRestoreModifiers(
            from.modifiers.clone(),
            KeyModifierFlags::new(),
            TYPE_UP,
        ),
    );
    seq
}

#[derive(Default)]
struct SharedState {
    transformer: Arc<XKBTransformer>,
    mappings: Mappings,
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

        match ev {
            EvdevInputEvent {
                event_code: EventCode::EV_KEY(KEY_BACKSPACE),
                value: 1,
                ..
            } => {
                self.window.pop();
            }
            EvdevInputEvent {
                event_code: EventCode::EV_KEY(KEY_DELETE),
                value: 1,
                ..
            } => {
                self.window.remove(0);
            }
            // key event
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

                if ev.value == 1 {
                    let key = shared_state.transformer.raw_to_utf(&key, &self.modifiers);

                    if let Some(key) = key {
                        self.window.push(key.chars().next().unwrap());
                        // TODO set window size dynamically
                        if self.window.len() > 32 {
                            self.window.remove(0);
                        }

                        let mut hit = None;

                        for i in (0..self.window.len()).rev() {
                            let search: String = self.window.iter().skip(i).collect();
                            if let Some(x) = shared_state.mappings.get(&search) {
                                hit = Some((x, search.len()));
                            }
                        }

                        if let Some((to, from_len)) = hit {
                            self.window.clear();
                            let next = match next {
                                Some(x) => x,
                                None => {
                                    return;
                                }
                            };

                            for _ in 1..from_len {
                                let _ = next
                                    .send(InputEvent::Raw(Key::from(KEY_BACKSPACE).to_input_ev(1)));
                                let _ = next
                                    .send(InputEvent::Raw(Key::from(KEY_BACKSPACE).to_input_ev(0)));
                            }

                            for action in to {
                                match action {
                                    RuntimeKeyAction::KeyAction(key_action) => {
                                        let ev = key_action.to_input_ev();
                                        let _ = next.send(InputEvent::Raw(ev));
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
                                        // events.append(&mut new_events);
                                        for ev in new_events {
                                            let _ = next.send(InputEvent::Raw(ev));
                                        }
                                    }
                                }
                            }
                            return;
                        }
                    }
                }

                event_handlers::update_modifiers(
                    &mut self.modifiers,
                    &KeyAction::from_input_ev(&ev),
                );
            }
            _ => {}
        }

        if let Some(next) = next {
            let _ = next.send(raw_ev);
        }
    }
}

#[pyclass]
pub struct TextMapper {
    pub id: Uuid,
    shared_state: Arc<RwLock<SharedState>>,
    transformer: Arc<XKBTransformer>,
    tmp_next: Mutex<Option<SubscriberNew>>,
}

#[pymethods]
impl TextMapper {
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

        let shared_state = Arc::new(RwLock::new(SharedState::default()));

        Ok(Self {
            id,
            shared_state,
            transformer,
            tmp_next: Default::default(),
        })
    }

    pub fn map(&mut self, from: String, to: String) -> PyResult<()> {
        if from.len() > 32 {
            return Err(PyRuntimeError::new_err(
                "'from' side cannot be longer than 32 character",
            ));
        }

        let from_seq: Vec<KeyClickActionWithMods> =
            parse_key_sequence(&from, Some(&self.transformer))
                .map_err(|err| {
                    PyRuntimeError::new_err(format!(
                        "mapping error on the 'from' side:\n{}",
                        ApplicationError::KeyParse(err.to_string()),
                    ))
                })?
                .into_iter()
                .map(|x| match x {
                    ParsedKeyAction::KeyClickAction(x) => Some(x),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| PyRuntimeError::new_err("invalid key sequence"))?;

        let to = parse_key_sequence(&to, Some(&self.transformer)).map_err(|err| {
            PyRuntimeError::new_err(format!(
                "mapping error on the 'to' side:\n{}",
                ApplicationError::KeySequenceParse(err.to_string()),
            ))
        })?;

        let to = _map(from_seq.last().unwrap(), to);
        self.shared_state.write().unwrap().mappings.insert(from, to);

        Ok(())
    }

    pub fn snapshot(
        &self,
        existing: Option<&TextMapperSnapshot>,
    ) -> PyResult<Option<TextMapperSnapshot>> {
        let mut shared_state = self.shared_state.write().unwrap();

        if let Some(existing) = existing {
            shared_state.mappings = existing.mappings.clone();
            return Ok(None);
        }

        Ok(Some(TextMapperSnapshot {
            mappings: shared_state.mappings.clone(),
        }))
    }
}

impl TextMapper {
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
            let mut state = State::default();
            loop {
                let ev = ev_rx.recv().await.unwrap();
                let shared_state = _shared_state.read().unwrap();

                state.handle(ev, next.as_ref(), &shared_state);
            }
        });

        ev_tx.clone()
    }
}

#[pyclass]
pub struct TextMapperSnapshot {
    mappings: Mappings,
}
