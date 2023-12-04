use pyo3::{PyAny, PyRefMut};

use crate::*;

// path hash, InputEvent
pub type SubscribeEvent = (u64, InputEvent);
pub type Subscriber = tokio::sync::mpsc::UnboundedSender<SubscribeEvent>;


pub fn add_event_subscription(target: &PyAny) -> Option<Subscriber> {
    if let Ok(target) = target.extract::<PyRefMut<Mapper>>() {
        return Some(target.subscribe());
    }
    if let Ok(target) = target.extract::<PyRefMut<TextMapper>>() {
        return Some(target.subscribe());
    }
    if let Ok(target) = target.extract::<PyRefMut<Writer>>() {
        return Some(target.subscribe())
    }
    None
}

#[macro_export]
macro_rules! linkable {
    () => {
        pub fn _link(&mut self, path: Vec<Uuid>, target: &PyAny) -> PyResult<()> {
            use crate::subscriber::*;

            if target.is_none() {
                self.subscriber.store(None);
                return Ok(());
            }

            let target = match add_event_subscription(target) {
                Some(target) => target,
                None => { return Err(PyRuntimeError::new_err("unsupported link target")); }
            };
            self.subscriber.store(Some(Arc::new(target)));
            Ok(())
        }
    };
}
