use crate::*;

const READER: &str = "reader";
const WRITER: &str = "writer";

#[test_main]
async fn single_key_click() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

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

#[test_main]
async fn hold_key() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

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

#[test_main]
async fn break_chord() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

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

#[test_main]
async fn simple_chord() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

        reader_send_all(py, m, READER, &keys("{a down}{b down}{a up}{b up}"));
        sleep(py, 55);
        assert_eq!(writer_read_all(py, m, WRITER), keys("c"),);
        sleep(py, 55);
        assert_empty!(py, m, WRITER);

        Ok(())
    })?;
    Ok(())
}

#[test_main]
async fn multi_chord() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

        reader_send_all(py, m, "reader", &keys("{a down}{b down}{b up}{b down}{a up}{b up}"));
        sleep(py, 55);
        assert_eq!(writer_read_all(py, m, WRITER), keys("cc"),);

        Ok(())
    })?;
    Ok(())
}

#[test_main]
async fn chord_to_function() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = &pytests::include_python!();

        let counter = m.getattr("counter").unwrap().extract::<i32>().unwrap();
        assert_eq!(counter, 0);

        reader_send_all(py, m, READER, &keys("{c down}{d down}{c up}{d up}"));
        sleep(py, 55);
        assert_empty!(py, m, WRITER);

        let counter = m.getattr("counter").unwrap().extract::<i32>().unwrap();
        assert_eq!(counter, 1);

        Ok(())
    })?;
    Ok(())
}
