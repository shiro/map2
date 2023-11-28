use std::thread;
use std::time::Duration;
use map2::key_primitives::Key;

use crate::*;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, "reader", &vec![
            Key::from_str("a").unwrap().to_input_ev(1),
            Key::from_str("a").unwrap().to_input_ev(0),
        ] );

        py.allow_threads(|| { thread::sleep(Duration::from_millis(25)); });

        assert_eq!(
            writer_read_all(py, m, "writer"),
            vec![
                Key::from_str("b").unwrap().to_input_ev(1),
                Key::from_str("b").unwrap().to_input_ev(0),
            ]
        );

        Ok(())
    })?;
    Ok(())
}