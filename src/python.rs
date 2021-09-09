use std::thread;

use evdev_rs::enums::{EV_KEY, EventType};
use evdev_rs::TimeVal;
use pyo3::prelude::*;

use crate::{EventCode, INPUT_EV_DUMMY_TIME, InputEvent};
use crate::*;
use crate::task::JoinHandle;
use anyhow::Error;
use crate::device::device_logging::print_event_debug;

#[pyclass]
struct PyKey {
    #[pyo3(get, set)]
    code: u32,
    #[pyo3(get, set)]
    value: i32,
}


#[pyclass]
struct InstanceHandle {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<()>,
}

impl InstanceHandle {
    pub fn new(exit_tx: oneshot::Sender<()>, join_handle: std::thread::JoinHandle<()>) -> Self {
        InstanceHandle { exit_tx, join_handle }
    }
}

#[pymethods]
impl InstanceHandle {
    pub fn map(&self) -> PyResult<()> {
        println!("wow, map");
        Ok(())
    }
}


#[pyfunction]
fn sum_as_string(py: Python, a: usize, b: usize, callback: PyObject) {
    //Ok((a + b).to_string())
    // println!("Hello from Rust!");

    let ev = PyKey { code: EV_KEY::KEY_L as u32, value: 1 };

    // let ev = PyInputEvent { 0: InputEvent::new(&INPUT_EV_DUMMY_TIME, &EventCode::EV_KEY(EV_KEY::BTN_0)), 1 };
    // let ev = PyInputEvent { 0: EventCode::EV_KEY(EV_KEY::BTN_0) };
    // let ev = &KEY_K;
    // let k = Key::from_str(&EventType::EV_KEY, "KEY_SLASH").unwrap()

    callback.call(py, (ev, ), None);
    // callback.call(py, (), None)
    // None
}


#[pyfunction]
fn setup(py: Python, callback: PyObject) -> PyResult<InstanceHandle> {
    let handle = _setup(callback).unwrap();
    Ok(handle)
}

#[pymodule]
fn map2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    m.add_function(wrap_pyfunction!(setup, m)?)?;

    Ok(())
}

struct Instance {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<()>,
}

lazy_static! {
    static ref INSTANCE_MAP: Mutex<HashMap<u32, Instance>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}

fn _setup(callback: PyObject) -> Result<InstanceHandle> {
    let (exit_tx, exit_rx) = oneshot::channel();
    let join_handle = thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let mut configuration = parse_cli().unwrap();

            // initialize global state
            // let mut stdout = io::stdout();
            // let mut state = State::new();
            // let mut window_cycle_token: usize = 0;
            // let mut mappings = CompiledKeyMappings::new();
            // let mut window_change_handlers = vec![];

            // add a small delay if run from TTY so we don't miss 'enter up' which is often released when the device is grabbed
            if atty::is(atty::Stream::Stdout) {
                thread::sleep(time::Duration::from_millis(300));
            }

            // initialize device communication channels
            let (ev_reader_init_tx, ev_reader_init_rx) = oneshot::channel();
            let (ev_writer_tx, mut ev_writer_rx) = mpsc::channel(128);

            bind_udev_inputs(&configuration.devices, ev_reader_init_tx, ev_writer_tx).await?;
            let mut ev_reader_tx = ev_reader_init_rx.await?;

            loop {
                let ev = ev_writer_rx.recv().await.unwrap();
                // print_event_debug(&ev);

                let code = match ev.event_code {
                    EventCode::EV_KEY(code) => code,
                    _ => continue,
                };

                let key = PyKey { code: code as u32, value: ev.value };
                {
                    use std::time::Instant;
                    let now = Instant::now();
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    callback.call(py, (key, ), None);

                    let elapsed = now.elapsed();
                    println!("Elapsed: {:.2?}", elapsed);
                }
            }

            exit_rx.await?;
            Ok::<(), anyhow::Error>(())
        }).unwrap();
    });

    // let instance = Instance {
    //     exit_tx,
    //     join_handle,
    // };

    // let handle = 0;
    let handle = InstanceHandle::new(exit_tx, join_handle);

    // let mut map = INSTANCE_MAP.lock().unwrap();
    // map.insert(handle, instance);

    // println!("done!");
    Ok(handle)
}