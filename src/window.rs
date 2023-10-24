use std::thread;

use pyo3::*;
use pyo3::prelude::*;

use crate::*;
use crate::time::Duration;

use hyprland::prelude::*;
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::dispatch::*;


#[pyclass]
pub struct Window {
    thread_handle: Option<thread::JoinHandle<()>>,
    thread_exit_tx: Option<oneshot::Sender<()>>,
    subscription_id_cnt: u32,
    subscriptions_tx: mpsc::Sender<ControlMessage>,
}

#[pymethods]
impl Window {
    #[new]
    pub fn new() -> Self {
        let (subscriptions_tx, x11_thread_handle, x11_thread_exit_tx) = spawn_x11_thread();

        Window {
            thread_handle: Some(x11_thread_handle),
            thread_exit_tx: Some(x11_thread_exit_tx),
            subscription_id_cnt: 0,
            subscriptions_tx,
        }
    }

    fn on_window_change(&mut self, callback: PyObject) -> WindowOnWindowChangeSubscription {
        let _ = self.subscriptions_tx.send(ControlMessage::Subscribe(self.subscription_id_cnt, callback));
        let subscription = WindowOnWindowChangeSubscription { id: self.subscription_id_cnt };
        self.subscription_id_cnt += 1;
        subscription
    }
    fn remove_on_window_change(&self, subscription: &WindowOnWindowChangeSubscription) {
        let _ = self.subscriptions_tx.send(ControlMessage::Unsubscribe(subscription.id));
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.thread_exit_tx.take().unwrap().send(());
        let _ = self.thread_handle.take().unwrap().try_timed_join(Duration::from_millis(100)).unwrap();
    }
}

#[pyclass]
struct WindowOnWindowChangeSubscription {
    id: u32,
}

pub enum ControlMessage {
    Subscribe(u32, PyObject),
    Unsubscribe(u32),
}

pub fn spawn_x11_thread() -> (mpsc::Sender<ControlMessage>, thread::JoinHandle<()>, oneshot::Sender<()>) {
    let (subscription_tx, subscription_rx) = mpsc::channel();
    let (exit_tx, exit_rx) = oneshot::channel();
    let handle = thread::spawn(move || {
        let mut subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let mut event_listener = EventListener::new();

        event_listener.add_active_window_change_handler(move |data, _| {
            if exit_rx.try_recv().is_ok() { return; }

            while let Ok(msg) = subscription_rx.try_recv() {
                match msg {
                    ControlMessage::Subscribe(id, callback) => { subscriptions.lock().unwrap().insert(id, callback); }
                    ControlMessage::Unsubscribe(id) => { subscriptions.lock().unwrap().remove(&id); }
                }
            }

            if let Some(val) = data {
                let val = ActiveWindowInfo {
                    class: val.window_class.clone(),
                    instance: "".to_string(),
                    name: val.window_title.clone(),
                };

                let gil = Python::acquire_gil();
                let py = gil.python();
                for callback in subscriptions.lock().unwrap().values() {
                    let is_callable = callback.cast_as::<PyAny>(py).map_or(false, |obj| obj.is_callable());
                    if !is_callable { continue; }
                    let _ = callback.call(py, (val.class.clone(), ), None);
                }
            }
        });
        let _ = event_listener.start_listener();
        ()
    });
    (subscription_tx, handle, exit_tx)
}