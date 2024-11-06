use crate::python::*;
use pyo3::{PyAny, PyRefMut};

use crate::mapper::*;
use crate::*;

pub fn node_to_link_dst(target: &PyAny) -> Option<Arc<dyn LinkDst>> {
    if let Ok(target) = target.extract::<PyRefMut<Mapper>>() {
        return Some(target.link.clone());
    }
    if let Ok(mut target) = target.extract::<PyRefMut<TextMapper>>() {
        return Some(target.link.clone());
    }
    if let Ok(target) = target.extract::<PyRefMut<ChordMapper>>() {
        return Some(target.link.clone());
    }
    if let Ok(target) = target.extract::<PyRefMut<ModifierMapper>>() {
        return Some(target.link.clone());
    }

    if let Ok(target) = target.extract::<PyRefMut<Writer>>() {
        return Some(target.link.clone());
    }
    None
}

pub fn node_to_link_src(target: &PyAny) -> Option<Arc<dyn LinkSrc>> {
    if let Ok(target) = target.extract::<PyRefMut<Reader>>() {
        return Some(target.link.clone());
    }

    if let Ok(target) = target.extract::<PyRefMut<Mapper>>() {
        return Some(target.link.clone());
    }
    if let Ok(mut target) = target.extract::<PyRefMut<TextMapper>>() {
        return Some(target.link.clone());
    }
    if let Ok(target) = target.extract::<PyRefMut<ChordMapper>>() {
        return Some(target.link.clone());
    }
    if let Ok(target) = target.extract::<PyRefMut<ModifierMapper>>() {
        return Some(target.link.clone());
    }
    None
}

pub trait LinkSrc: Send + Sync {
    fn id(&self) -> &Uuid;
    fn link_to(&self, node: Arc<dyn LinkDst>) -> Result<()>;
    fn unlink_to(&self, id: &Uuid) -> Result<bool>;
}

pub trait LinkDst: Send + Sync {
    fn id(&self) -> &Uuid;
    fn link_from(&self, node: Arc<dyn LinkSrc>) -> Result<()>;
    fn unlink_from(&self, id: &Uuid) -> Result<bool>;
    fn send(&self, ev: InputEvent) -> Result<()>;
}

pub trait SubscriberHashmapExt {
    fn send_all(&self, ev: InputEvent);
}

impl SubscriberHashmapExt for HashMap<Uuid, Arc<dyn LinkDst>> {
    fn send_all(&self, ev: InputEvent) {
        self.values().for_each(|link| {
            // TODO handle err
            link.send(ev.clone());
        });
    }
}

pub trait SubscriberVecExt {
    fn send_all(&self, ev: InputEvent);
}

impl SubscriberVecExt for Vec<Arc<dyn LinkDst>> {
    fn send_all(&self, ev: InputEvent) {
        self.iter().for_each(|link| {
            link.send(ev.clone());
        });
    }
}
