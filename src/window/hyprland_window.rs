use crate::python::*;
use crate::window::window_base::{ActiveWindowInfo, WindowControlMessage, WindowHandler};
use crate::*;
use hyprland::async_closure;
use hyprland::data::Client;
use hyprland::data::{Monitor, Workspace};
use hyprland::event_listener::WindowEventData;
use hyprland::event_listener::{AsyncEventListener, EventListener};
use hyprland::shared::HyprDataActive;
use hyprland::shared::{HyprData, HyprDataActiveOptional};
use std::panic::catch_unwind;

pub fn hyprland_window_handler() -> WindowHandler {
    Box::new(
        |exit_rx: oneshot::Receiver<()>,
         mut subscription_rx: tokio::sync::mpsc::Receiver<WindowControlMessage>|
         -> Result<()> {
            let subscriptions: Arc<Mutex<HashMap<u32, Py<PyAny>>>> = Arc::new(Mutex::new(HashMap::new()));

            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_info| {}));

            let mut event_listener = catch_unwind(|| AsyncEventListener::new()).map_err(|err| {
                anyhow!(
                    "hyprland connection error: {}",
                    err.downcast::<String>().unwrap_or(Box::new("unknown".to_string()))
                )
            })?;

            std::panic::set_hook(prev_hook);

            let handle_window_change = {
                let subscriptions = subscriptions.clone();
                move |info: ActiveWindowInfo| {
                    Python::with_gil(|py| {
                        for callback in subscriptions.lock().unwrap().values() {
                            let is_callable = callback.as_ref(py).is_callable();
                            if !is_callable {
                                continue;
                            }

                            let ret = callback.call(py, (info.class.clone(),), None);

                            if let Err(err) = ret {
                                eprintln!("{err}");
                                std::process::exit(1);
                            }
                        }
                    });
                }
            };

            event_listener.add_active_window_change_handler(move |info| {
                Box::pin({
                    let handle_window_change = handle_window_change.clone();
                    async move {
                        let info = info.unwrap();
                        handle_window_change(ActiveWindowInfo {
                            class: info.window_class,
                            instance: "".to_string(),
                            name: info.window_title,
                        });
                    }
                })
            });

            pyo3_asyncio::tokio::get_runtime().block_on(async move {
                tokio::task::spawn(async move {
                    event_listener.start_listener_async().await;
                });

                tokio::task::spawn(async move {
                    loop {
                        let msg = subscription_rx.recv().await.unwrap();
                        match msg {
                            WindowControlMessage::Subscribe(id, callback) => {
                                subscriptions.lock().unwrap().insert(id, callback.clone());

                                if let Ok(Some(info)) = Client::get_active() {
                                    Python::with_gil(|py| {
                                        let is_callable = callback.as_ref(py).is_callable();
                                        //if !is_callable { continue; }

                                        let ret = callback.call(py, (info.class.clone(),), None);

                                        if let Err(err) = ret {
                                            eprintln!("{err}");
                                            std::process::exit(1);
                                        }
                                    });
                                }
                            }
                            WindowControlMessage::Unsubscribe(id) => {}
                        }
                    }
                });
            });

            Ok(())
        },
    )
}
