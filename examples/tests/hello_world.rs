use crate::*;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = pytests::include_python!();

        assert_eq!(
            writer_read_all(py, m, "writer"),
            vec![
                Key::from_str("leftshift").unwrap().to_input_ev(1),
                Key::from_str("h").unwrap().to_input_ev(1),
                Key::from_str("h").unwrap().to_input_ev(0),
                Key::from_str("leftshift").unwrap().to_input_ev(0),
                Key::from_str("e").unwrap().to_input_ev(1),
                Key::from_str("e").unwrap().to_input_ev(0),
                Key::from_str("l").unwrap().to_input_ev(1),
                Key::from_str("l").unwrap().to_input_ev(0),
                Key::from_str("l").unwrap().to_input_ev(1),
                Key::from_str("l").unwrap().to_input_ev(0),
                Key::from_str("o").unwrap().to_input_ev(1),
                Key::from_str("o").unwrap().to_input_ev(0),
                Key::from_str("space").unwrap().to_input_ev(1),
                Key::from_str("space").unwrap().to_input_ev(0),
                Key::from_str("w").unwrap().to_input_ev(1),
                Key::from_str("w").unwrap().to_input_ev(0),
                Key::from_str("o").unwrap().to_input_ev(1),
                Key::from_str("o").unwrap().to_input_ev(0),
                Key::from_str("r").unwrap().to_input_ev(1),
                Key::from_str("r").unwrap().to_input_ev(0),
                Key::from_str("l").unwrap().to_input_ev(1),
                Key::from_str("l").unwrap().to_input_ev(0),
                Key::from_str("d").unwrap().to_input_ev(1),
                Key::from_str("d").unwrap().to_input_ev(0),
                Key::from_str("leftshift").unwrap().to_input_ev(1),
                Key::from_str("1").unwrap().to_input_ev(1),
                Key::from_str("1").unwrap().to_input_ev(0),
                Key::from_str("leftshift").unwrap().to_input_ev(0),
            ]
        );

        Ok(())
    })?;
    Ok(())
}
