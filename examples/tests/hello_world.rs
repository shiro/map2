use crate::*;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        assert_eq!(writer_read_all(py, m, "writer"), keys("Hello world!"),);

        Ok(())
    })?;
    Ok(())
}
