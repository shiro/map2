use pyo3::{PyAny, PyRefMut};

use crate::*;

pub trait Subscribable {
    fn handle(&self, id: &str, ev: InputEvent);
}

pub type Subscriber = Arc<dyn Subscribable + Send + Sync>;


pub fn add_event_subscription(target: &PyAny) -> Option<Subscriber> {
    if let Ok(mut target) = target.extract::<PyRefMut<KeyMapper>>() {
        return Some(target.subscribe());
    }
    if let Ok(mut target) = target.extract::<PyRefMut<MotionMapper>>() {
        return Some(target.subscribe())
    }
    if let Ok(mut target) = target.extract::<PyRefMut<Writer>>() {
        return Some(target.subscribe())
    }
    None
}

#[macro_export]
macro_rules! linkable {
    () => {
        pub fn _link(&mut self, target: &PyAny) -> PyResult<()> {
            use crate::subscriber::*;

            let target = match add_event_subscription(target) {
                Some(target) => target,
                None => { return Err(PyRuntimeError::new_err("unsupported link target")); }
            };
            self.subscriber.store(Some(Arc::new(target)));
            Ok(())
        }
    };
}
