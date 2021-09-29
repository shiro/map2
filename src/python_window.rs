use crate::*;
use pyo3::*;
use std::thread;
use crate::time::Duration;
use pyo3::prelude::*;

#[pyclass]
pub struct Window {
    x11_thread_handle: std::thread::JoinHandle<()>,
    subscription_id_cnt: u32,
    subscriptions_tx: mpsc::UnboundedSender<X11ControlMessage>,
}

#[pymethods]
impl Window {
    #[new]
    pub fn new() -> Self {
        let (subscriptions_tx, x11_thread_handle) = spawn_x11_thread();
        Window { x11_thread_handle, subscription_id_cnt: 0, subscriptions_tx }
    }

    fn on_window_change(&mut self, callback: PyObject) -> WindowOnWindowChangeSubscription {
        let _ = self.subscriptions_tx.send(X11ControlMessage::Subscribe(self.subscription_id_cnt, callback));
        let subscription = WindowOnWindowChangeSubscription { id: self.subscription_id_cnt };
        self.subscription_id_cnt += 1;
        subscription
    }
    fn remove_on_window_change(&self, subscription: &WindowOnWindowChangeSubscription) {
        let _ = self.subscriptions_tx.send(X11ControlMessage::Unsubscribe(subscription.id));
    }
}

#[pyclass]
struct WindowOnWindowChangeSubscription {
    id: u32,
}

pub enum X11ControlMessage {
    Subscribe(u32, PyObject),
    Unsubscribe(u32),
}

pub fn spawn_x11_thread() -> (mpsc::UnboundedSender<X11ControlMessage>, thread::JoinHandle<()>) {
    let (subscription_tx, mut subscription_rx) = mpsc::unbounded_channel();
    let handle = thread::spawn(move || {
        let x11_state = Arc::new(x11_initialize().unwrap());
        let mut subscriptions = HashMap::new();

        loop {
            loop {
                if let Ok(msg) = subscription_rx.try_recv() {
                    match msg {
                        X11ControlMessage::Subscribe(id, callback) => { subscriptions.insert(id, callback); }
                        X11ControlMessage::Unsubscribe(id) => { subscriptions.remove(&id); }
                    }
                } else { break; }
            }

            let res = get_window_info_x11(&x11_state);

            if let Ok(Some(val)) = res {
                let gil = Python::acquire_gil();
                let py = gil.python();
                for callback in subscriptions.values() {
                    let is_callable = callback.cast_as::<PyAny>(py).map_or(false, |obj| obj.is_callable());

                    if !is_callable { continue; }

                    let _ = callback.call(py, (val.class.clone(), ), None);
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    });
    (subscription_tx, handle)
}
