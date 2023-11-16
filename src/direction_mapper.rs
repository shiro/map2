use std::sync::RwLock;
use evdev_rs::enums::EV_REL;

use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::impl_::wrap::OkWrap;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::*;
use crate::event::InputEvent;
use crate::event_loop::{EventLoop, PythonArgument};
use crate::parsing::key_action::*;
use crate::parsing::python::*;
use crate::python::*;
use crate::subscriber::Subscriber;
use crate::writer::Writer;
use crate::xkb::UTFToRawInputTransformer;

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


pub struct DirectionMapperInner {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    relative_handler: RwLock<Option<PyObject>>,
}

impl DirectionMapperInner {
    pub fn handle(&self, id: &str, raw_ev: InputEvent) {
        if let Some(subscriber) = self.subscriber.load().deref() {
            let ev = match &raw_ev { InputEvent::Raw(ev) => ev };

            if let Some(relative_handler) = self.relative_handler.read().unwrap().as_ref() {
                match ev {
                    EvdevInputEvent {
                        event_code: EventCode::EV_REL(key),
                        value,
                        ..
                    } => {
                        let name = match key {
                            EV_REL::REL_X => "REL_X",
                            EV_REL::REL_Y => "REL_Y",
                            _ => {return;}
                        }.to_string();

                        EVENT_LOOP.lock().unwrap().execute(relative_handler.clone(), Some(vec![
                            PythonArgument::String(name),
                            PythonArgument::Number(*value),
                        ]));
                    }
                    &_ => {}
                }
                return;
            }

            subscriber.handle("", raw_ev);
        }
    }
}


#[pyclass]
pub struct DirectionMapper {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    pub inner: Arc<DirectionMapperInner>,
}

#[pymethods]
impl DirectionMapper {
    #[new]
    #[pyo3(signature = (* * _kwargs))]
    pub fn new(_kwargs: Option<&PyDict>) -> PyResult<Self> {
        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));

        let inner = Arc::new(DirectionMapperInner {
            subscriber: subscriber.clone(),
            relative_handler: RwLock::new(None),
        });

        Ok(Self {
            subscriber,
            inner,
        })
    }

    pub fn map_relative(&mut self, py: Python, relative_handler: PyObject) -> PyResult<()> {
        let is_callable = relative_handler.as_ref(py).is_callable();

        if is_callable {
            *self.inner.relative_handler.write().unwrap() = Some(relative_handler);
            return Ok(());
        }

        Err(PyRuntimeError::new_err("expected a callable object"))
    }

    pub fn link(&mut self, target: &PyAny) {
        if let Ok(mut target) = target.extract::<PyRefMut<Writer>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Writer(target.inner.clone())))
            );
        } else if let Ok(mut target) = target.extract::<PyRefMut<Mapper>>() {
            self.subscriber.store(
                Some(Arc::new(Subscriber::Mapper(target.inner.clone())))
            );
        }
    }
}