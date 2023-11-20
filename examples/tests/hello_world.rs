use map2::*;
use map2::python::*;
use map2::testing;

#[pyo3_asyncio::tokio::test]
async fn hello_world() -> PyResult<()> {
    Python::with_gil(|py| -> PyResult<()> {
        let m = PyModule::from_code(py, pytests::include_python!(), "", "")?;

        let map2 = m.getattr("map2")?;
        let res = map2.getattr("__test")?.call0()?;
        let s: Vec<String> = res.extract()?;
        let s: Vec<testing::TestEvent> = s.into_iter().map(|x| serde_json::from_str(&x).unwrap()).collect();
        assert_eq!(s, vec![
            testing::TestEvent::WriterOutEv(EvdevInputEvent::new(
                &evdev_rs::TimeVal { tv_sec: 0, tv_usec: 0 },
                &KEY_H.event_code,
                1,
            )),
            testing::TestEvent::WriterOutEv(EvdevInputEvent::new(
                &evdev_rs::TimeVal { tv_sec: 0, tv_usec: 0 },
                &KEY_H.event_code,
                0,
            )),
        ]);

        Ok(())
    })?;
    Ok(())
}