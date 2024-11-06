use std::thread;
use std::time::Duration;

use crate::*;

#[test_main]
async fn key_passthrough() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("a"));

        py.allow_threads(|| {
            thread::sleep(Duration::from_millis(25));
        });

        assert_eq!(writer_read_all(py, m, "writer"), keys("a"),);

        Ok(())
    })?;
    Ok(())
}

#[test_main]
async fn modifier_key_click() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("{capslock down}a{capslock up}"));

        py.allow_threads(|| {
            thread::sleep(Duration::from_millis(25));
        });

        assert_eq!(writer_read_all(py, m, "writer"), keys("b"),);

        Ok(())
    })?;
    Ok(())
}

#[test_main]
async fn modifier_passthrough() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("{capslock}"));

        py.allow_threads(|| {
            thread::sleep(Duration::from_millis(25));
        });

        assert_eq!(writer_read_all(py, m, "writer"), keys("{capslock}"),);

        Ok(())
    })?;
    Ok(())
}
