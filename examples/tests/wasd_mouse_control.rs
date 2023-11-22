use std::thread;
use std::time::Duration;
use evdev_rs::enums::{EV_REL, EventCode};

use crate::*;

#[pyo3_asyncio::tokio::test]
async fn wasd_mouse_control() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send(py, m, "reader_kbd", &EvdevInputEvent::new(&Default::default(), &KEY_W.event_code, 1));

        // sleep for long enough to trigger the timeout once
        py.allow_threads(|| { thread::sleep(Duration::from_millis(25)); });

        reader_send(py, m, "reader_kbd", &EvdevInputEvent::new(&Default::default(), &KEY_W.event_code, 0));

        assert_eq!(
            writer_read_all(py, m, "writer_mouse"),
            vec![
                EvdevInputEvent::new(&Default::default(), &EventCode::EV_REL(EV_REL::REL_Y), -15),
                EvdevInputEvent::new(&Default::default(), &EventCode::EV_REL(EV_REL::REL_Y), -15),
            ]
        );

        Ok(())
    })?;
    Ok(())
}