use super::suffix_tree::SuffixTree;
use super::*;
use crate::mapper::mapping_functions::*;
use crate::mapper::RuntimeKeyAction;
use crate::python::*;
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};
use crate::*;
use nom::Slice;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use ApplicationError::TooManyEvents;

type Mappings = SuffixTree<RuntimeAction>;

#[derive(Default)]
struct State {
    transformer: Arc<XKBTransformer>,
    prev: HashMap<Uuid, Arc<dyn LinkSrc>>,
    next: HashMap<Uuid, Arc<dyn LinkDst>>,
    mappings: Mappings,
    modifiers: Arc<KeyModifierState>,
    window: Vec<char>,
}

#[pyclass]
pub struct TextMapper {
    pub id: Uuid,
    pub link: Arc<TextMapperLink>,
    ev_tx: tokio::sync::mpsc::Sender<InputEvent>,
    state: Arc<Mutex<State>>,
}

#[pymethods]
impl TextMapper {
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
        let link = Arc::new(TextMapperLink { id, ev_tx: ev_tx.clone(), state: state.clone() });

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
        if from.len() > 32 {
            return Err(PyRuntimeError::new_err("'from' side cannot be longer than 32 character"));
        }

        let from_seq: Vec<KeyClickActionWithMods> = parse_key_sequence(&from, Some(&state.transformer))
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

        let to = if to.as_ref(py).is_callable() {
            RuntimeAction::PythonCallback(Default::default(), Arc::new(to))
        } else {
            let to = to.extract::<String>(py).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::InvalidInputType { type_: "String".to_string() }
                ))
            })?;
            let to = parse_key_sequence(&to, Some(&state.transformer)).map_err(|err| {
                PyRuntimeError::new_err(format!(
                    "mapping error on the 'to' side:\n{}",
                    ApplicationError::KeySequenceParse(err.to_string()),
                ))
            })?;

            let mut to: Vec<RuntimeKeyAction> =
                to.to_key_actions().into_iter().map(|action| RuntimeKeyAction::KeyAction(action)).collect();

            RuntimeAction::ActionSequence(to)
        };

        state.mappings.insert(from, to);

        Ok(())
    }

    pub fn snapshot(&self, existing: Option<&TextMapperSnapshot>) -> Option<TextMapperSnapshot> {
        let mut state = self.state.blocking_lock();
        if let Some(existing) = existing {
            state.mappings = existing.mappings.clone();
            return None;
        }
        Some(TextMapperSnapshot { mappings: state.mappings.clone() })
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

impl Drop for TextMapper {
    fn drop(&mut self) {
        self.unlink_from_all();
        self.unlink_to_all();
    }
}

#[derive(Clone)]
pub struct TextMapperLink {
    id: Uuid,
    ev_tx: Sender<InputEvent>,
    state: Arc<Mutex<State>>,
}

impl LinkSrc for TextMapperLink {
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

impl LinkDst for TextMapperLink {
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
pub struct TextMapperSnapshot {
    mappings: Mappings,
}

fn _map(from: &KeyClickActionWithMods, to: Vec<ParsedKeyAction>) -> Vec<RuntimeKeyAction> {
    let mut seq: Vec<RuntimeKeyAction> =
        to.to_key_actions().into_iter().map(|action| RuntimeKeyAction::KeyAction(action)).collect();
    seq.insert(0, RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), KeyModifierFlags::new(), TYPE_UP));
    seq
}

async fn handle(_state: Arc<Mutex<State>>, raw_ev: InputEvent) {
    let mut state = _state.lock().await;
    let mut state = &mut *state;
    if !state.next.is_empty() {
        state.next.send_all(raw_ev.clone());
    }
    let ev = match raw_ev {
        InputEvent::Raw(ev) => ev,
    };

    match ev {
        EvdevInputEvent { event_code: EventCode::EV_KEY(KEY_BACKSPACE), value: 1, .. } => {
            state.window.pop();
        }
        EvdevInputEvent { event_code: EventCode::EV_KEY(KEY_DELETE), value: 1, .. } => {
            state.window.remove(0);
        }
        // key event
        EvdevInputEvent { event_code: EventCode::EV_KEY(key), value, .. } => {
            if ev.value == 0 {
                let key = state.transformer.raw_to_utf(&key, &state.modifiers);

                if let Some(key) = key {
                    state.window.push(key.chars().next().unwrap());
                    // TODO set window size dynamically
                    if state.window.len() > 32 {
                        state.window.remove(0);
                    }

                    let mut hit = None;

                    for i in (0..state.window.len()).rev() {
                        let search: String = state.window.iter().skip(i).collect();
                        if let Some(x) = state.mappings.get(&search) {
                            hit = Some((x, search.len()));
                        }
                    }

                    if let Some((to, from_len)) = hit {
                        state.window.clear();

                        if !state.next.is_empty() {
                            for _ in 0..from_len {
                                state.next.send_all(InputEvent::Raw(Key::from(KEY_BACKSPACE).to_input_ev(1)));
                                state.next.send_all(InputEvent::Raw(Key::from(KEY_BACKSPACE).to_input_ev(0)));
                            }
                        }

                        match to {
                            RuntimeAction::ActionSequence(seq) => {
                                for action in seq {
                                    match action {
                                        RuntimeKeyAction::KeyAction(key_action) => {
                                            state.next.send_all(InputEvent::Raw(key_action.to_input_ev()));
                                        }
                                        RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                            if !state.next.is_empty() {
                                                let new_events = release_restore_modifiers(
                                                    &state.modifiers,
                                                    &from_flags,
                                                    &to_flags,
                                                    &to_type,
                                                );
                                                for ev in new_events {
                                                    state.next.send_all(InputEvent::Raw(ev));
                                                }
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
                                    for ev in new_events {
                                        state.next.send_all(InputEvent::Raw(ev));
                                    }
                                }
                                // delay the callback until the backspace events are processed
                                tokio::time::sleep(Duration::from_millis(10 * from_len as u64)).await;
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

                        // return after handled match
                        return;
                    }
                }
            }

            event_handlers::update_modifiers(&mut state.modifiers, &KeyAction::from_input_ev(&ev));
        }
        _ => {}
    }
}
