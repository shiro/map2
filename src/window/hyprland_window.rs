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
                    tokio::task::spawn_blocking(move || {
                        Python::with_gil(|py| {
                            let subscriptions = {
                                subscriptions.lock().unwrap().values().map(|v| v.bind(py)).cloned().collect::<Vec<_>>()
                            };
                            for callback in subscriptions {
                                let is_callable = callback.is_callable();
                                if !is_callable {
                                    continue;
                                }

                                let ret = callback.call((info.class.clone(),), None);

                                if let Err(err) = ret {
                                    eprintln!("{err}");
                                    std::process::exit(1);
                                }
                            }
                        });
                    });
                }
            };

            event_listener.add_active_window_changed_handler(move |info| {
                Box::pin({
                    let handle_window_change = handle_window_change.clone();
                    async move {
                        let info = info.unwrap();
                        handle_window_change(ActiveWindowInfo {
                            class: info.class,
                            instance: "".to_string(),
                            name: info.title,
                        });
                    }
                })
            });

            tokio::task::spawn(async move {
                event_listener.start_listener_async().await;
            });

            tokio::task::spawn(async move {
                loop {
                    let msg = match subscription_rx.recv().await {
                        Some(v) => v,
                        None => return,
                    };
                    match msg {
                        WindowControlMessage::Subscribe(id, callback) => {
                            Python::with_gil(|py| {
                                subscriptions.lock().unwrap().insert(id, callback.clone_ref(py));
                            });

                            if let Ok(Some(info)) = Client::get_active_async().await {
                                println!(" --> w1");
                                //if !is_callable { continue; }

                                tokio::task::spawn_blocking(move || {
                                    Python::with_gil(|py| {
                                        println!(" --> w1 start");
                                        let is_callable = callback.bind(py).is_callable();
                                        let ret = callback.call_bound(py, (info.class.clone(),), None);
                                        if let Err(err) = ret {
                                            eprintln!("{err}");
                                            std::process::exit(1);
                                        }
                                        println!(" --> w1 done");
                                    });
                                });
                            }
                        }
                        WindowControlMessage::Unsubscribe(id) => {}
                    }
                }
            });

            Ok(())
        },
    )
}
