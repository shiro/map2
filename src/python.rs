use evdev_rs::enums::{EV_KEY, EventType};
use evdev_rs::TimeVal;
use pyo3::prelude::*;

use crate::{EventCode, INPUT_EV_DUMMY_TIME, InputEvent};
use crate::*;

#[pyclass]
struct PyKey {
    #[pyo3(get, set)]
    code: u32,
    #[pyo3(get, set)]
    value: i32,
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


#[pymodule]
fn map2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    Ok(())
}
