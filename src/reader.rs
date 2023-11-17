use ::oneshot;

use crate::*;
use crate::python::*;
use crate::event::InputEvent;
use crate::subscriber::{Subscriber, add_event_subscription};


#[pyclass]
pub struct Reader {
    reader_exit_tx: Option<oneshot::Sender<()>>,
    reader_thread_handle: Option<thread::JoinHandle<Result<()>>>,
    subscriber: Arc<ArcSwapOption<Subscriber>>,
}


#[pymethods]
impl Reader {
    #[new]
    #[pyo3(signature = (* * kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = kwargs
            .ok_or_else(|| PyRuntimeError::new_err("no options provided"))?
            .extract()
            .map_err(|_| PyRuntimeError::new_err("the options object must be a dict"))?;

        let patterns: Vec<&str> = options.get("patterns")
            .ok_or_else(|| PyRuntimeError::new_err("'patterns' is required but was not provided"))?
            .extract()
            .map_err(|_| PyRuntimeError::new_err("'patterns' must be a list"))?;

        if patterns.is_empty() {
            return Err(PyRuntimeError::new_err("the pattern list cannot be empty"));
        }

        let (reader_exit_tx, reader_exit_rx) = oneshot::channel();

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));
        let _subscriber = subscriber.clone();

        let handler = Arc::new(move |id: &str, ev: EvdevInputEvent| {
            if let Some(subscriber) = _subscriber.load().deref() {
                subscriber.handle(id, InputEvent::Raw(ev));
            }
        });

        let reader_thread_handle = grab_udev_inputs(&patterns, handler, reader_exit_rx).map_err(err_to_py)?;

        Ok(Self {
            reader_exit_tx: Some(reader_exit_tx),
            reader_thread_handle: Some(reader_thread_handle),
            subscriber,
        })
    }

    pub fn link(&mut self, target: &PyAny) -> PyResult<()> { self._link(target) }
}

impl Reader {
    linkable!();
}