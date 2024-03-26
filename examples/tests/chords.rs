use crate::*;

const READER: &str = "reader";
const WRITER: &str = "writer";

#[pyo3_asyncio::tokio::test]
async fn single_key_click() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, READER, &keys("a"));
        sleep(py, 55);
        assert_keys!(py, m, WRITER, "a");

        reader_send_all(py, m, READER, &keys("b"));
        sleep(py, 55);
        assert_keys!(py, m, WRITER, "b");

        Ok(())
    })?;
    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn hold_key() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, READER, &keys("{a down}"));
        sleep(py, 55);
        reader_send_all(py, m, READER, &keys("{a repeat}{a up}"));
        sleep(py, 10);
        assert_keys!(py, m, WRITER, "{a down}{a repeat}{a up}");
        sleep(py, 55);
        assert_empty!(py, m, WRITER);

        Ok(())
    })?;
    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn break_chord() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, READER, &keys("{a down}"));
        sleep(py, 10);
        reader_send_all(py, m, READER, &keys("{z down}"));
        sleep(py, 10);
        assert_keys!(py, m, WRITER, "a{z down}");
        reader_send_all(py, m, READER, &keys("{a up}{z up}"));
        sleep(py, 10);
        assert_keys!(py, m, WRITER, "{z up}");

        Ok(())
    })?;
    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn simple_chord() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("{a down}{b down}{a up}{b up}"));
        sleep(py, 55);
        assert_eq!(writer_read_all(py, m, "writer"), keys("c"),);
        sleep(py, 55);
        assert_empty!(py, m, WRITER);

        Ok(())
    })?;
    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn multi_chord() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("{a down}{b down}{b up}{b down}{a up}{b up}"));
        sleep(py, 55);
        assert_eq!(writer_read_all(py, m, "writer"), keys("cc"),);

        Ok(())
    })?;
    Ok(())
}
