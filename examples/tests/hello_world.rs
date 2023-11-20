use crate::*;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        assert_eq!(
            (
                writer_read(py, m, "writer"),
                writer_read(py, m, "writer"),
            ),
            (
                Some(EvdevInputEvent::new(&Default::default(), &KEY_H.event_code, 1)),
                Some(EvdevInputEvent::new(&Default::default(), &KEY_H.event_code, 0)),
            )
        );

        Ok(())
    })?;
    Ok(())
}