use std::hash::{Hash, Hasher};
use ::oneshot;

use crate::*;
use crate::python::*;
use crate::event::InputEvent;
use crate::subscriber::{Subscriber, add_event_subscription};


#[pyclass]
pub struct Reader {
    pub id: Arc<Uuid>,
    subscriber: Arc<ArcSwapOption<Subscriber>>,
    #[cfg(not(feature = "integration"))]
    reader_exit_tx: Option<oneshot::Sender<()>>,
    #[cfg(not(feature = "integration"))]
    reader_thread_handle: Option<thread::JoinHandle<Result<()>>>,
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

        #[cfg(not(feature = "integration"))]
            let (reader_exit_tx, reader_exit_rx) = oneshot::channel();

        let subscriber: Arc<ArcSwapOption<Subscriber>> = Arc::new(ArcSwapOption::new(None));
        let _subscriber = subscriber.clone();

        let id = Arc::new(Uuid::new_v4());

        let _id = id.clone();

        let mut h = DefaultHasher::new();
        vec![_id.clone()].hash(&mut h);
        let path_hash = h.finish();

        let handler = Arc::new(move |_: &str, ev: EvdevInputEvent| {
            if let Some(subscriber) = _subscriber.load().deref() {
                let _ = subscriber.send((path_hash, InputEvent::Raw(ev)));
            }
        });

        #[cfg(not(feature = "integration"))]
            let reader_thread_handle = grab_udev_inputs(&patterns, handler, reader_exit_rx).map_err(err_to_py)?;

        // #[cfg(not(feature = "integration"))]
        //     let reader_thread_handle = pyo3_asyncio::tokio::get_runtime().block_on(async move {
        //     // tokio::time::sleep(Duration::from_millis(1000)).await;
        //     // println!("hi future");
        //     grab_udev_inputs(&patterns, handler, reader_exit_rx).map_err(err_to_py)
        // })?;


        Ok(Self {
            id,
            subscriber,
            #[cfg(not(feature = "integration"))]
            reader_exit_tx: Some(reader_exit_tx),
            #[cfg(not(feature = "integration"))]
            reader_thread_handle: Some(reader_thread_handle),
        })
    }

    #[cfg(feature = "integration")]
    pub fn send(&mut self, ev: String) -> PyResult<()> {
        let ev: EvdevInputEvent = serde_json::from_str(&ev).unwrap();

        let mut h = DefaultHasher::new();
        vec![self.id.clone()].hash(&mut h);
        let path_hash = h.finish();

        if let Some(subscriber) = self.subscriber.load().deref() {
            let _ = subscriber.send((path_hash, InputEvent::Raw(ev)));
        };
        Ok(())
    }
}

impl Reader {
    pub fn _link(&mut self, target: &PyAny) -> PyResult<()> {
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
}