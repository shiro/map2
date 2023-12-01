#[cfg(not(feature = "integration"))]
use std::sync::mpsc;
#[cfg(not(feature = "integration"))]
use std::sync::mpsc::TryRecvError;
#[cfg(not(feature = "integration"))]
use evdev_rs::enums::EventType::EV_SYN;

use python::*;

use crate::*;
use crate::device::*;
use crate::device::virt_device::DeviceCapabilities;
use crate::subscriber::{SubscribeEvent, Subscriber};
use crate::xkb::XKBTransformer;
use crate::xkb_transformer_registry::{TransformerParams, XKB_TRANSFORMER_REGISTRY};


#[pyclass]
pub struct Writer {
    ev_tx: Subscriber,
    exit_tx: tokio::sync::mpsc::UnboundedSender<()>,
    transformer: Arc<XKBTransformer>,
    #[cfg(feature = "integration")]
    ev_rx: tokio::sync::mpsc::UnboundedReceiver<SubscribeEvent>,
}

#[pymethods]
impl Writer {
    #[new]
    #[pyo3(signature = (* * kwargs))]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let options: HashMap<&str, &PyAny> = match kwargs {
            Some(py_dict) => py_dict.extract()?,
            None => HashMap::new()
        };

        let device_name = match options.get("name") {
            Some(option) => option.extract::<String>()
                .map_err(|_| PyRuntimeError::new_err("'name' must be a string"))?,
            None => "Virtual map2 output".to_string()
        };

        let mut capabilities = DeviceCapabilities::new();
        if let Some(_capabilities) = options.get("capabilities") {
            let _capabilities: HashMap<&str, &PyAny> = _capabilities.extract()
                .map_err(|_| PyRuntimeError::new_err("the 'capabilities' object must be a dict"))?;

            if _capabilities.contains_key("keyboard") { capabilities.enable_keyboard(); }
            if _capabilities.contains_key("buttons") { capabilities.enable_buttons(); }
            if _capabilities.contains_key("rel") { capabilities.enable_rel(); }
            if _capabilities.contains_key("abs") { capabilities.enable_abs(); }
        } else {
            capabilities.enable_keyboard();
            capabilities.enable_buttons();
            capabilities.enable_rel();
        }

        let device_init_policy = match options.get("clone_from") {
            Some(_existing_dev_fd) => {
                let existing_dev_fd = _existing_dev_fd.extract::<String>()
                    .map_err(|_| PyRuntimeError::new_err("the 'clone_from' option must be a string"))?;

                if options.get("capabilities").is_some() {
                    return Err(PyRuntimeError::new_err("expected only one of: 'clone_from', 'capabilities'"));
                }

                virtual_output_device::DeviceInitPolicy::CloneExistingDevice(existing_dev_fd)
            }
            None => {
                virtual_output_device::DeviceInitPolicy::NewDevice(device_name, capabilities)
            }
        };

        let kbd_model = options.get("model").and_then(|x| x.extract().ok());
        let kbd_layout = options.get("layout").and_then(|x| x.extract().ok());
        let kbd_variant = options.get("variant").and_then(|x| x.extract().ok());
        let kbd_options = options.get("options").and_then(|x| x.extract().ok());
        let transformer = XKB_TRANSFORMER_REGISTRY
            .get(&TransformerParams::new(kbd_model, kbd_layout, kbd_variant, kbd_options))
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

        let (ev_tx, mut ev_rx) = tokio::sync::mpsc::unbounded_channel::<SubscribeEvent>();
        let (exit_tx, mut exit_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        #[cfg(not(feature = "integration"))]
        {
            // grab udev device
            let mut output_device = virtual_output_device::init_virtual_output_device(&device_init_policy)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
            get_runtime().spawn(async move {
                loop {
                    loop {
                        let (_, ev) = ev_rx.recv().await.unwrap();

                        if let Ok(()) = exit_rx.try_recv() { return; }

                        let ev = match &ev { InputEvent::Raw(ev) => ev };
                        let mut syn = SYN_REPORT.clone();
                        syn.time.tv_sec = ev.time.tv_sec;
                        syn.time.tv_usec = ev.time.tv_usec;


                        #[cfg(not(feature = "integration"))]
                        {
                            let _ = output_device.send(&ev);
                            let _ = output_device.send(&syn);
                        }

                        // this is a hack that stops successive events to not get registered
                        if let EventCode::EV_KEY(_) = ev.event_code {
                            tokio::time::sleep(Duration::from_millis(1)).await;
                        }
                    }
                }
            });
        }

        let handle = Self {
            ev_tx,
            exit_tx,
            transformer,
            #[cfg(feature = "integration")]
            ev_rx,
        };

        Ok(handle)
    }

    pub fn send(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence(val.as_str(), Some(&self.transformer))
            .map_err(|err|
                ApplicationError::KeySequenceParse(err.to_string()).into_py()
            )?
            .to_key_actions();

        for action in actions {
            let _ = self.ev_tx.send((0, InputEvent::Raw(action.to_input_ev())));
        }
        Ok(())
    }

    #[cfg(feature = "integration")]
    pub fn __test__read_ev(&mut self) -> PyResult<Option<String>> {
        match self.ev_rx.try_recv().ok() {
            Some((_, ev)) => {
                let ev = match ev { InputEvent::Raw(ev) => { ev } };
                Ok(Some(serde_json::to_string(&ev).unwrap()))
            }
            None => { Ok(None) }
        }
    }
}

impl Writer {
    pub fn subscribe(&self) -> Subscriber {
        self.ev_tx.clone()
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        let _ = self.exit_tx.send(());
        let _ = self.ev_tx.send((0, InputEvent::Raw(SYN_REPORT.clone())));
    }
}