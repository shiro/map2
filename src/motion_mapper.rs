use std::sync::RwLock;

use evdev_rs::enums::EV_REL;

use crate::*;
use crate::event::InputEvent;
use crate::event_loop::PythonArgument;
use crate::python::*;
use crate::subscriber::{add_event_subscription, Subscribable, Subscriber};

#[derive(Default)]
struct Inner {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    relative_handler: RwLock<Option<PyObject>>,
    fallback_handler: RwLock<Option<PyObject>>,
}

impl Subscribable for Inner {
    fn handle(&self, id: &str, raw_ev: InputEvent) {
        let ev = match &raw_ev { InputEvent::Raw(ev) => ev };

        if let EvdevInputEvent { event_code: EventCode::EV_REL(key), value, .. } = ev {
            match key {
                EV_REL::REL_X | EV_REL::REL_Y => {
                    if let Some(handler) = self.relative_handler.read().unwrap().as_ref() {
                        // let name = if *key == EV_REL::REL_X { "X" } else { "Y" }.to_string();
                        let name = format!("{key:?}").to_string();

                        EVENT_LOOP.lock().unwrap().execute(handler.clone(), Some(vec![
                            PythonArgument::String(name),
                            PythonArgument::Number(*value),
                        ]));
                        return;
                    }
                }
                _ => {}
            }
        }

        if let Some(handler) = self.fallback_handler.read().unwrap().as_ref() {
            let name = format!("{:?}", ev.event_code).to_string();
            EVENT_LOOP.lock().unwrap().execute(handler.clone(), Some(vec![
                PythonArgument::String(name),
                PythonArgument::Number(ev.value),
            ]));
        }

        if let Some(subscriber) = self.subscriber.load().deref() {
            subscriber.handle("", raw_ev);
        }
    }
}


#[pyclass]
pub struct MotionMapper {
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    inner: Arc<Inner>,
}

#[pymethods]
impl MotionMapper {
    #[new]
    #[pyo3(signature = (* * _kwargs))]
    pub fn new(_kwargs: Option<&PyDict>) -> PyResult<Self> {
        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));

        let inner = Arc::new(Inner {
            subscriber: subscriber.clone(),
            ..Default::default()
        });

        Ok(Self {
            subscriber,
            inner,
        })
    }

    pub fn map_relative(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        if handler.as_ref(py).is_callable() {
            *self.inner.relative_handler.write().unwrap() = Some(handler);
            return Ok(());
        }
        Err(InputError::NotCallable.into())
    }

    pub fn map_fallback(&mut self, py: Python, handler: PyObject) -> PyResult<()> {
        if handler.as_ref(py).is_callable() {
            *self.inner.fallback_handler.write().unwrap() = Some(handler);
            return Ok(());
        }
        Err(InputError::NotCallable.into())
    }

    pub fn link(&mut self, target: &PyAny) -> PyResult<()> { self._link(target) }

    pub fn snapshot(&self, existing: Option<&MotionMapperSnapshot>) -> PyResult<Option<MotionMapperSnapshot>> {
        if let Some(existing) = existing {
            *self.inner.relative_handler.write().unwrap() = existing.relative_handler.clone();
            return Ok(None);
        }

        Ok(Some(MotionMapperSnapshot {
            relative_handler: self.inner.relative_handler.read().unwrap().clone(),
        }))
    }
}

impl MotionMapper {
    linkable!();

    pub fn subscribe(&self) -> Subscriber {
        self.inner.clone()
    }
}

#[pyclass]
pub struct MotionMapperSnapshot {
    relative_handler: Option<PyObject>,
}