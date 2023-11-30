use std::thread;
use std::time::Duration;

use crate::*;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("a"));

        py.allow_threads(|| { thread::sleep(Duration::from_millis(25)); });

        assert_eq!(
            writer_read_all(py, m, "writer"),
            keys("b"),
        );

        Ok(())
    })?;
    Ok(())
}