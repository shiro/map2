use crate::python::*;
use crate::window::window_base::{ActiveWindowInfo, WindowControlMessage, WindowHandler};
use crate::*;
use hyprland::data::Client;
use hyprland::event_listener::EventListener;
use hyprland::shared::{HyprData, HyprDataActiveOptional};
use std::panic::catch_unwind;

pub fn hyprland_window_handler() -> WindowHandler {
    Box::new(|exit_rx: oneshot::Receiver<()>, subscription_rx: mpsc::Receiver<WindowControlMessage>| -> Result<()> {
        let subscriptions = Arc::new(Mutex::new(HashMap::new()));

        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_info| {}));

        let mut event_listener = catch_unwind(|| EventListener::new()).map_err(|err| {
            anyhow!(
                "hyprland connection error: {}",
                err.downcast::<String>().unwrap_or(Box::new("unknown".to_string()))
            )
        })?;

        std::panic::set_hook(prev_hook);

        event_listener.add_active_window_change_handler(move |info| {
            if exit_rx.try_recv().is_ok() {
                return;
            }

            // let msg = subscription_rx.recv().unwrap();
            // match msg {
            //     WindowControlMessage::Subscribe(id, callback) => {
            //         subscriptions.lock().unwrap().insert(id, callback.clone());

            //         let class = Client::get_active().expect("a").expect("b").class.clone();
            //         Python::with_gil(|py| {
            //             let is_callable = callback.as_ref(py).is_callable(); //if !is_callable { continue; }

            //             let ret = callback.call(py, (class,), None);

            //             if let Err(err) = ret {
            //                 eprintln!("{err}");
            //                 std::process::exit(1);
            //             }
            //         });
            //     }
            //     WindowControlMessage::Unsubscribe(id) => {}
            // }

            while let Ok(msg) = subscription_rx.try_recv() {
                match msg {
                    WindowControlMessage::Subscribe(id, callback) => {
                        subscriptions.lock().unwrap().insert(id, callback);
                    }
                    WindowControlMessage::Unsubscribe(id) => {
                        subscriptions.lock().unwrap().remove(&id);
                    }
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
                        if !is_callable {
                            continue;
                        }

                        let ret = callback.call(py, (val.class.clone(),), None);

                        if let Err(err) = ret {
                            eprintln!("{err}");
                            std::process::exit(1);
                        }
                    }
                });
            }
        });
        event_listener.start_listener()?;
        Ok(())
    })
}
