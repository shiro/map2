use crate::*;
use crate::python::*;

#[pyclass]
pub struct ReaderInstanceHandle {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<()>,
}
