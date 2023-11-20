use map2::python::*;



#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        PyModule::from_code(py, pytests::include_python!(), "", "")?;

        Ok(())
    })?;
    Ok(())
}
