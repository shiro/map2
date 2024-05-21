use pyo3::{PyAny, PyRefMut};

use crate::*;

pub type SubscriberNew = tokio::sync::mpsc::UnboundedSender<InputEvent>;
