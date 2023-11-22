use std::panic::catch_unwind;
use crate::*;
use crate::python::*;
use hyprland::event_listener::EventListenerMutable as EventListener;
use crate::window::window_base::{ActiveWindowInfo, WindowControlMessage, WindowHandler};

pub fn hyprland_window_handler() -> WindowHandler {
    Box::new(|exit_rx: oneshot::Receiver<()>,
        subscription_rx: mpsc::Receiver<WindowControlMessage>| -> Result<()> {
        let mut subscriptions = Arc::new(Mutex::new(HashMap::new()));

        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_info| {}));

        let mut event_listener = catch_unwind(|| EventListener::new())
            .map_err(|err| anyhow!(
                    "hyprland connection error: {}",
                    err.downcast::<String>().unwrap_or(Box::new("unknown".to_string())
            )))?;

        std::panic::set_hook(prev_hook);

        event_listener.add_active_window_change_handler(move |info, _| {
            if exit_rx.try_recv().is_ok() { return; }

            while let Ok(msg) = subscription_rx.try_recv() {
                match msg {
                    WindowControlMessage::Subscribe(id, callback) => { subscriptions.lock().unwrap().insert(id, callback); }
                    WindowControlMessage::Unsubscribe(id) => { subscriptions.lock().unwrap().remove(&id); }
                }
            }

            if let Some(info) = info {
                let val = ActiveWindowInfo {
                    class: info.window_class.clone(),
                    instance: "".to_string(),
                    name: info.window_title.clone(),
                };

                Python::with_gil(|py| {
                    for callback in subscriptions.lock().unwrap().values() {
                        let is_callable = callback.as_ref(py).is_callable();
                        if !is_callable { continue; }

                        let _ = callback.call(py, (val.class.clone(), ), None);
                    }
                });
            }
        });
        event_listener.start_listener()?;
        Ok(())
    })
}