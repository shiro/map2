use crate::python::*;
use crate::window::window_base::{ActiveWindowInfo, WindowControlMessage, WindowEventType, WindowHandler};
use crate::*;
use hyprland::data::Client;
use hyprland::data::{Monitor, Workspace};
use hyprland::event_listener::WindowEventData;
use hyprland::event_listener::{AsyncEventListener, EventListener};
use hyprland::shared::HyprDataActive;
use hyprland::shared::{HyprData, HyprDataActiveOptional};
use std::panic::catch_unwind;
use tokio::sync::oneshot;
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, BufReader};

pub fn hyprland_window_handler() -> WindowHandler {
    Box::new(
        |exit_rx: oneshot::Receiver<()>,
         mut subscription_rx: tokio::sync::mpsc::Receiver<WindowControlMessage>|
         -> Result<()> {
            let subscriptions: Arc<Mutex<HashMap<u32, (Py<PyAny>, WindowEventType)>>> = Arc::new(Mutex::new(HashMap::new()));
            let prev_window: Arc<Mutex<Option<ActiveWindowInfo>>> = Arc::new(Mutex::new(None));

            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_info| {}));

            let mut event_listener = catch_unwind(|| AsyncEventListener::new()).map_err(|err| {
                anyhow!(
                    "hyprland connection error: {}",
                    err.downcast::<String>().unwrap_or(Box::new("unknown".to_string()))
                )
            })?;

            std::panic::set_hook(prev_hook);

            let handle_window_change: Arc<dyn Fn(Option<ActiveWindowInfo>) + Send + Sync> = Arc::new({
                let subscriptions = subscriptions.clone();
                let prev_window = prev_window.clone();
                move |info: Option<ActiveWindowInfo>| {
                    let subscriptions = subscriptions.clone();
                    let prev_window = prev_window.clone();
                    tokio::task::spawn_blocking(move || {
                        let callbacks_to_call = {
                            Python::with_gil(|py| {
                                subscriptions.lock().unwrap().iter().map(|(_, (cb, evt))| (cb.clone_ref(py), *evt)).collect::<Vec<_>>()
                            })
                        };

                        // Extract state under lock, then release before calling Python
                        let old_window_for_blur = {
                            let mut prev = prev_window.lock().unwrap();
                            match &info {
                                Some(new_window) => {
                                    *prev = Some(new_window.clone());
                                    None
                                }
                                None => prev.take()
                            }
                        };

                        Python::with_gil(|py| {
                            match &info {
                                Some(new_window) => {
                                    // Focus event - call focus handlers
                                    for (callback, event_type) in &callbacks_to_call {
                                        if *event_type != WindowEventType::Focus {
                                            continue;
                                        }
                                        let is_callable = callback.bind(py).is_callable();
                                        if !is_callable {
                                            continue;
                                        }

                                        let window_dict = PyDict::new(py);
                                        let _ = window_dict.set_item("class", &new_window.class);
                                        let _ = window_dict.set_item("instance", &new_window.instance);
                                        let _ = window_dict.set_item("title", &new_window.title);

                                        let ret = callback.call(py, (window_dict,), None);

                                        if let Err(err) = ret {
                                            eprintln!("{err}");
                                            std::process::exit(1);
                                        }
                                    }
                                }
                                None => {
                                    // Blur event - call blur handlers if we had a previous window
                                    if let Some(old_window) = old_window_for_blur {
                                        for (callback, event_type) in &callbacks_to_call {
                                            if *event_type != WindowEventType::Blur {
                                                continue;
                                            }
                                            let is_callable = callback.bind(py).is_callable();
                                            if !is_callable {
                                                continue;
                                            }

                                            let window_dict = PyDict::new(py);
                                            let _ = window_dict.set_item("class", &old_window.class);
                                            let _ = window_dict.set_item("instance", &old_window.instance);
                                            let _ = window_dict.set_item("title", &old_window.title);

                                            let ret = callback.call(py, (window_dict,), None);

                                            if let Err(err) = ret {
                                                eprintln!("{err}");
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    });
                }
            });

            let hwc_handler = handle_window_change.clone();
            event_listener.add_active_window_changed_handler(move |info| {
                Box::pin({
                    let handle_window_change = hwc_handler.clone();
                    async move {
                        let window_info = info.map(|info| ActiveWindowInfo {
                            class: info.class,
                            instance: "".to_string(),
                            title: info.title,
                        });
                        handle_window_change(window_info);
                    }
                })
            });

            // Listen to raw IPC socket for activewindowv2 to detect blur
            let handle_window_change_ipc = handle_window_change.clone();
            tokio::task::spawn(async move {
                    loop {
                        let socket_path = format!(
                            "{}/hypr/{}/.socket2.sock",
                            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string()),
                            std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default()
                        );
                        match UnixStream::connect(&socket_path).await {
                            Ok(stream) => {
                                let reader = BufReader::new(stream);
                                let mut lines = reader.lines();
                                
                                while let Ok(Some(line)) = lines.next_line().await {
                                    if line.starts_with("activewindowv2>>") {
                                        let addr = line.strip_prefix("activewindowv2>>").unwrap_or("").to_string();
                                        // If address is empty, no window is active
                                        if addr.is_empty() {
                                            handle_window_change_ipc(None);
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Connection error, will retry
                            }
                        }
                        // Reconnect on socket error
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
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
                        WindowControlMessage::Subscribe(id, callback, event_type) => {
                            Python::with_gil(|py| {
                                subscriptions.lock().unwrap().insert(id, (callback.clone_ref(py), event_type));
                            });

                            if event_type == WindowEventType::Focus {
                                if let Ok(Some(info)) = Client::get_active_async().await {
                                    let window_info = ActiveWindowInfo {
                                        class: info.class,
                                        instance: "".to_string(),
                                        title: info.title,
                                    };

                                    tokio::task::spawn_blocking(move || {
                                        Python::with_gil(|py| {
                                            let is_callable = callback.bind(py).is_callable();
                                            if !is_callable {
                                                return;
                                            }

                                             let window_dict = PyDict::new(py);
                                             let _ = window_dict.set_item("class", &window_info.class);
                                             let _ = window_dict.set_item("instance", &window_info.instance);
                                             let _ = window_dict.set_item("title", &window_info.title);

                                             let ret = callback.call(py, (window_dict,), None);
                                            if let Err(err) = ret {
                                                eprintln!("{err}");
                                                std::process::exit(1);
                                            }
                                        });
                                    });
                                }
                            }
                        }
                        WindowControlMessage::Unsubscribe(id) => {
                            subscriptions.lock().unwrap().remove(&id);
                        }
                    }
                }
            });

            Ok(())
        },
    )
}
