use map2::key_primitives::Key;
use crate::*;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        assert_eq!(
            writer_read_all(py, m, "writer"),
            vec![
                Key::from_str("h").unwrap().to_input_ev(1),
                Key::from_str("h").unwrap().to_input_ev(0),
            ]
        );

        Ok(())
    })?;
    Ok(())
}