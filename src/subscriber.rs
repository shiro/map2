use crate::python::*;
use pyo3::{PyAny, PyRefMut};

use crate::mapper::*;
use crate::*;

pub fn node_to_link_dst(target: &PyBound<PyAny>) -> Option<Arc<dyn LinkDst>> {
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

pub fn node_to_link_src(target: &PyBound<PyAny>) -> Option<Arc<dyn LinkSrc>> {
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
    fn py_object(&self) -> Arc<PyObject>;
    fn clear_next(&self) -> Vec<Arc<dyn LinkDst>>;
    fn next(&self) -> Vec<Arc<dyn LinkDst>>;
}

pub trait LinkSrcExt {
    fn py_insert_after(self, target: &PyBound<PyAny>) -> PyResult<()>;
    fn py_link_to(self, target: &PyBound<PyAny>) -> PyResult<()>;
    fn py_unlink_to(self, target: &PyBound<PyAny>) -> PyResult<bool>;
    fn py_unlink_to_all(self);
}

impl LinkSrcExt for Arc<dyn LinkSrc> {
    fn py_link_to(self, target: &PyBound<PyAny>) -> PyResult<()> {
        let target_dst =
            node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a \"destination\" node"))?;
        target_dst.link_from(self.clone());
        self.link_to(target_dst);
        Ok(())
    }

    fn py_unlink_to(self, target: &PyBound<PyAny>) -> PyResult<bool> {
        let target_dst =
            node_to_link_dst(target).ok_or_else(|| PyRuntimeError::new_err("expected a \"destination\" node"))?;
        target_dst.unlink_from(self.id());
        self.unlink_to(target_dst.id()).map_err(err_to_py)
    }

    fn py_unlink_to_all(self) {
        for node in self.clear_next() {
            let _ = node.unlink_from(self.id());
        }
    }

    fn py_insert_after(self, target: &PyBound<PyAny>) -> PyResult<()> {
        let target_src = node_to_link_src(target)
            .ok_or_else(|| PyRuntimeError::new_err("expected a \"source & destination\" node"))?;
        let _ = node_to_link_dst(target)
            .ok_or_else(|| PyRuntimeError::new_err("expected a \"source & destination\" node"))?;

        // move this node's dst nodes the target's dst nodes
        for node in self.clear_next() {
            target_src.link_to(node);
        }

        self.py_link_to(target);
        Ok(())
    }
}

pub trait LinkDst: Send + Sync {
    fn id(&self) -> &Uuid;
    fn link_from(&self, node: Arc<dyn LinkSrc>) -> Result<()>;
    fn unlink_from(&self, id: &Uuid) -> Result<bool>;
    fn send(&self, ev: InputEvent) -> Result<()>;
    fn py_object(&self) -> Arc<PyObject>;
    fn clear_prev(&self) -> Vec<Arc<dyn LinkSrc>>;
    fn prev(&self) -> Vec<Arc<dyn LinkSrc>>;
}

pub trait LinkDstExt {
    fn py_insert_before(self, target: &PyBound<PyAny>) -> PyResult<()>;
    fn py_link_from(self, target: &PyBound<PyAny>) -> PyResult<()>;
    fn py_unlink_from(self, target: &PyBound<PyAny>) -> PyResult<bool>;
    fn py_unlink_from_all(self);
}

impl LinkDstExt for Arc<dyn LinkDst> {
    fn py_link_from(self, target: &PyBound<PyAny>) -> PyResult<()> {
        let target_src =
            node_to_link_src(target).ok_or_else(|| PyRuntimeError::new_err("expected a \"source\" node"))?;
        target_src.link_to(self.clone());
        self.link_from(target_src);
        Ok(())
    }

    fn py_unlink_from(self, target: &PyBound<PyAny>) -> PyResult<bool> {
        let target_src =
            node_to_link_src(target).ok_or_else(|| PyRuntimeError::new_err("expected a \"source\" node"))?;
        target_src.unlink_to(self.id());
        self.unlink_from(target_src.id()).map_err(err_to_py)
    }

    fn py_unlink_from_all(self) {
        for node in self.clear_prev() {
            let _ = node.unlink_to(self.id());
        }
    }

    fn py_insert_before(self, target: &PyBound<PyAny>) -> PyResult<()> {
        let _ = node_to_link_src(target)
            .ok_or_else(|| PyRuntimeError::new_err("expected a \"source & destination\" node"))?;
        let target_dst = node_to_link_dst(target)
            .ok_or_else(|| PyRuntimeError::new_err("expected a \"source & destination\" node"))?;

        // move this node's src nodes the target's src nodes
        for node in self.clear_prev() {
            target_dst.link_from(node);
        }

        self.py_link_from(target);
        Ok(())
    }
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

// pub trait InputEvVecExt {
//     fn send_all<Next: IntoIterator<Item = dyn LinkDst>>(self, next: &Next);
// }
//
// impl InputEvVecExt for Vec<evdev_rs::InputEvent> {
//     fn send_all<Next: IntoIterator<Item = dyn LinkDst>>(self, next: &Vec<Arc<dyn LinkDst>>) {
//         for ev in self {
//             // TODO handle err
//             let _ = next.send_all(InputEvent::Raw(ev));
//         }
//     }
// }

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
