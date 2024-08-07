use crate::platform::{get_platform, Platform};
use crate::python::*;
use crate::window::hyprland_window::hyprland_window_handler;
use crate::window::x11_window::x11_window_handler;
use crate::*;

#[derive(Debug, Clone)]
pub struct ActiveWindowInfo {
    pub class: String,
    pub instance: String,
    pub name: String,
}

pub type WindowHandler =
    Box<dyn Fn(oneshot::Receiver<()>, tokio::sync::mpsc::Receiver<WindowControlMessage>) -> Result<()> + Send + Sync>;

#[pyclass]
pub struct Window {
    thread_handle: Option<thread::JoinHandle<()>>,
    thread_exit_tx: Option<oneshot::Sender<()>>,
    subscription_id_cnt: u32,
    subscriptions_tx: tokio::sync::mpsc::Sender<WindowControlMessage>,
}

#[pymethods]
impl Window {
    #[new]
    pub fn new() -> Self {
        let handler = match get_platform() {
            Platform::Hyprland => hyprland_window_handler(),
            Platform::X11 => x11_window_handler(),
            Platform::Unknown => {
                eprintln!("{}", ApplicationError::UnsupportedPlatform);
                std::process::exit(1);
            }
        };

        let (subscriptions_tx, thread_handle, thread_exit_tx) = spawn_listener_thread(handler);

        Window {
            thread_handle: Some(thread_handle),
            thread_exit_tx: Some(thread_exit_tx),
            subscription_id_cnt: 0,
            subscriptions_tx,
        }
    }

    fn on_window_change(&mut self, callback: PyObject) -> WindowOnWindowChangeSubscription {
        let _ = futures::executor::block_on(
            self.subscriptions_tx.send(WindowControlMessage::Subscribe(self.subscription_id_cnt, callback)),
        );
        let subscription = WindowOnWindowChangeSubscription { id: self.subscription_id_cnt };
        self.subscription_id_cnt += 1;
        subscription
    }
    fn remove_on_window_change(&self, subscription: &WindowOnWindowChangeSubscription) {
        let _ = self.subscriptions_tx.send(WindowControlMessage::Unsubscribe(subscription.id));
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.thread_exit_tx.take().unwrap().send(());
        let _ = self.thread_handle.take().unwrap()/*.try_timed_join(Duration::from_millis(100)).unwrap()*/;
    }
}

#[pyclass]
struct WindowOnWindowChangeSubscription {
    id: u32,
}

pub enum WindowControlMessage {
    Subscribe(u32, PyObject),
    Unsubscribe(u32),
}

pub fn spawn_listener_thread(
    handler: WindowHandler,
) -> (tokio::sync::mpsc::Sender<WindowControlMessage>, thread::JoinHandle<()>, oneshot::Sender<()>) {
    let (subscription_tx, subscription_rx) = tokio::sync::mpsc::channel(255);
    let (exit_tx, exit_rx) = oneshot::channel();
    let handle = thread::spawn(move || {
        if let Err(err) = handler(exit_rx, subscription_rx) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    });
    (subscription_tx, handle, exit_tx)
}
