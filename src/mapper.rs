use std::sync::RwLock;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::event::InputEvent;
use crate::event_loop::EventLoop;
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::reader::Subscriber;
use crate::writer::Writer;

lazy_static! {
    static ref EVENT_LOOP: Mutex<EventLoop> = Mutex::new(EventLoop::new());
}


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


pub struct MapperInner {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    state_map: RwLock<HashMap<String, RwLock<State>>>,
    mappings: RwLock<Mappings>,
}

impl MapperInner {
    pub fn handle(&self, id: String, ev: InputEvent) {
        if let Some(subscriber) = self.subscriber.load().deref() {
            let mut state_map = self.state_map.read().unwrap();
            let mut state_guard = state_map.get(&id);
            if state_guard.is_none() {
                drop(state_map);
                let mut writable_state_map = self.state_map.write().unwrap();
                writable_state_map.insert(id.clone(), RwLock::new(State::new()));
                drop(writable_state_map);
                state_map = self.state_map.read().unwrap();
                state_guard = state_map.get(&id);
            }

            let mappings = self.mappings.read().unwrap();
            let mut state = state_guard.unwrap().write().unwrap();

            // start
            let ev = match ev { InputEvent::Raw(ev) => ev };

            let mut from_modifiers = KeyModifierFlags::new();
            from_modifiers.ctrl = state.modifiers.is_ctrl();
            from_modifiers.alt = state.modifiers.is_alt();
            from_modifiers.shift = state.modifiers.is_shift();
            from_modifiers.meta = state.modifiers.is_meta();

            let from_key_action = KeyActionWithMods {
                key: Key { event_code: ev.event_code },
                value: ev.value,
                modifiers: from_modifiers,
            };

            if let Some(runtime_action) = mappings.get(&from_key_action) {
                match runtime_action {
                    RuntimeAction::ActionSequence(seq) => {
                        for action in seq {
                            match action {
                                RuntimeKeyAction::KeyAction(key_action) => {
                                    let ev = key_action.to_input_ev();

                                    subscriber.handle("".to_string(), InputEvent::Raw(ev));
                                    subscriber.handle("".to_string(), InputEvent::Raw(SYN_REPORT.clone()));
                                }
                                RuntimeKeyAction::ReleaseRestoreModifiers(from_flags, to_flags, to_type) => {
                                    let mut new_events = release_restore_modifiers(&mut state, from_flags, to_flags, to_type);
                                    // events.append(&mut new_events);
                                    for ev in new_events {
                                        subscriber.handle("".to_string(), InputEvent::Raw(ev));
                                    }
                                }
                            }
                        }
                    }
                    RuntimeAction::PythonCallback(from_modifiers, callback_object) => {
                        // always release all trigger mods before running the callback
                        let mut new_events = release_restore_modifiers(&mut state, from_modifiers, &KeyModifierFlags::new(), &TYPE_UP);
                        for ev in new_events {
                            subscriber.handle("".to_string(), InputEvent::Raw(ev));
                        }

                        EVENT_LOOP.lock().unwrap().execute(callback_object.clone());
                    }
                    RuntimeAction::NOP => {}
                }

                return;
            }

            event_handlers::update_modifiers(&mut state, &KeyAction::from_input_ev(&ev));
            // end


            subscriber.handle("".to_string(), InputEvent::Raw(ev));
        }
    }
}


#[pyclass]
pub struct Mapper {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    pub inner: Arc<MapperInner>,

}

#[pymethods]
impl Mapper {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract().unwrap(),
            None => HashMap::new()
        };

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));

        let inner = Arc::new(MapperInner {
            subscriber: subscriber.clone(),
            state_map: RwLock::new(HashMap::new()),
            mappings: RwLock::new(Mappings::new()),
        });

        Ok(Self {
            subscriber,
            inner,
        })
    }

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        if let Ok(to) = to.extract::<String>(py) {
            let from = parse_key_action_with_mods_py(&from).unwrap();
            let to = parse_key_sequence_py(&to).unwrap();

            self._map_key(from, to)?;
            return Ok(());
        }

        let is_callable = to.cast_as::<PyAny>(py)
            .map_or(false, |obj| obj.is_callable());

        if is_callable {
            self._map_callback(from, to)?;
            return Ok(());
        }

        Err(PyTypeError::new_err("unknown type"))
    }

    pub fn map_key(&mut self, py: Python, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();
        let to = parse_key_action_with_mods_py(&to).unwrap();
        self._map_key(from, vec![to])?;

        Ok(())
    }

    pub fn link(&mut self, target: &PyAny) {
        if let Ok(mut target) = target.extract::<PyRefMut<Writer>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Writer(target.inner.clone())))
            );
        }else if let Ok(mut target) = target.extract::<PyRefMut<Mapper>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Mapper(target.inner.clone())))
            );
        }
    }
}

impl Mapper {
    fn _map_callback(&mut self, from: String, to: PyObject) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();

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
        }

        Ok(())
    }

    fn _map_key(&mut self, from: ParsedKeyAction, mut to: Vec<ParsedKeyAction>) -> PyResult<()> {
        match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    // action to click
                    if let ParsedKeyAction::KeyClickAction(to) = to {
                        let mapping = map_action_to_click(&from, &to);
                        self.inner.mappings.write().unwrap().insert(mapping.0, mapping.1);
                        return Ok(());
                    }
                    // action to action
                    if let ParsedKeyAction::KeyAction(to) = to {
                        let mapping = map_action_to_action(&from, &to);
                        self.inner.mappings.write().unwrap().insert(mapping.0, mapping.1);
                        return Ok(());
                    }
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
                        // click to action
                        ParsedKeyAction::KeyAction(to) => {
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
        }

        Ok(())
    }
}