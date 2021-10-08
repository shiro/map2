use core::mem;
use std::sync::mpsc;

use ::oneshot;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict};

use crate::*;

#[pyclass]
pub struct EventReader {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<Result<()>>,
    ev_rx: Option<mpsc::Receiver<InputEvent>>,
}


#[pymethods]
impl EventReader {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = kwargs
            .ok_or_else(|| PyTypeError::new_err("no options provided"))?
            .extract()
            .map_err(|_| PyTypeError::new_err("the options object must be a dict"))?;

        let patterns: Vec<&str> = options.get("patterns")
            .ok_or_else(|| PyTypeError::new_err("'patterns' is required but was not provided"))?
            .extract()
            .map_err(|_| PyTypeError::new_err("'patterns' must be a list"))?;

        let (exit_tx, exit_rx) = oneshot::channel();
        let (ev_tx, ev_rx) = mpsc::channel();

        let join_handle = grab_udev_inputs(&patterns, ev_tx, exit_rx)
            .map_err(|err| PyTypeError::new_err(err.to_string()))?;

        let handle = Self {
            exit_tx,
            join_handle,
            ev_rx: Some(ev_rx),
        };

        Ok(handle)
    }
}

impl EventReader {
    pub fn route(&mut self) -> Result<mpsc::Receiver<InputEvent>> {
        if self.ev_rx.is_none() {
            return Err(anyhow!("reader is already bound to an output, multiplexing is not allowed."));
        }
        let mut reader = None;
        mem::swap(&mut reader, &mut self.ev_rx);
        Ok(reader.unwrap())
    }
}
