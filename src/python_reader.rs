use core::mem;
use std::thread;
use crate::*;
use crate::python::*;
use std::sync::mpsc;
use ::oneshot;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

#[pyclass]
pub struct EventReader {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<Result<()>>,
    ev_rx: Option<mpsc::Receiver<InputEvent>>,
}


#[pymethods]
impl EventReader {
    #[new]
    pub fn new() -> PyResult<Self> {
        let (exit_tx, exit_rx) = oneshot::channel();
        let (ev_tx, mut ev_rx) = mpsc::channel();
        let mut configuration = parse_cli().unwrap();

        let join_handle = grab_udev_inputs(&configuration.devices, ev_tx, exit_rx)
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
