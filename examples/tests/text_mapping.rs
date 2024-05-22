use crate::*;

const READER: &str = "reader";
const WRITER: &str = "writer";

#[pyo3_asyncio::tokio::test]
async fn passes_through_unrealated_sequences() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, READER, &keys("hellp"));
        sleep(py, 5);
        assert_keys!(py, m, WRITER, "hellp");
        sleep(py, 5);
        assert_empty!(py, m, WRITER);

        Ok(())
    })?;
    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn hold_key() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, READER, &keys("hello"));
        sleep(py, 5);
        let mut output = "hello".to_owned();
        for _ in 0..("hello").len() {
            output.push_str("{backspace}");
        }
        output.push_str("bye");
        assert_keys!(py, m, WRITER, &output);
        sleep(py, 5);
        assert_empty!(py, m, WRITER);

        Ok(())
    })?;
    Ok(())
}
