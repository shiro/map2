#![feature(internal_output_capture)]

use std::io::Write;
use std::thread;
use std::time::Duration;

use map2::python::*;
use map2::*;

#[pyo3_asyncio::tokio::main]
async fn main() -> pyo3::PyResult<()> {
    let cmd = std::process::Command::new("maturin")
        .arg("dev")
        // .arg("--")
        // .arg("--cfg").arg("test")
        // .arg("--cfg").arg("integration")
        .arg("--features")
        .arg("integration")
        .output()?;

    if !cmd.status.success() {
        std::io::stderr().write(&cmd.stderr)?;
        std::process::exit(1);
    }

    pyo3_asyncio::testing::main().await
}

#[path = "../"]
mod integration_tests {
    automod::dir!("examples/tests");
}

pub fn writer_read(py: Python, module: &PyModule, name: &str) -> Option<EvdevInputEvent> {
    let target = module.getattr(name).unwrap().to_object(py);

    target
        .call_method0(py, "__test__read_ev")
        .unwrap()
        .extract::<Option<String>>(py)
        .unwrap()
        .and_then(|x| serde_json::from_str(&x).unwrap())
}

pub fn writer_read_all(py: Python, module: &PyModule, name: &str) -> Vec<EvdevInputEvent> {
    let mut acc = vec![];
    while let Some(ev) = writer_read(py, module, name) {
        acc.push(ev);
    }
    acc
}

pub fn reader_send(py: Python, module: &PyModule, name: &str, ev: &EvdevInputEvent) {
    let target = module.getattr(name).unwrap().to_object(py);
    let ev = serde_json::to_string(ev).unwrap();

    target.call_method(py, "__test__write_ev", (ev,), None).unwrap();
}

pub fn sleep(py: Python, millis: u64) {
    py.allow_threads(|| {
        thread::sleep(Duration::from_millis(millis));
    });
}

#[macro_export]
macro_rules! assert_keys {
    ($py: expr, $m: expr, $name: expr, $input: expr) => {
        assert_eq!(writer_read_all($py, $m, $name), keys($input),);
    };
}

#[macro_export]
macro_rules! assert_empty {
    ($py: expr, $module: expr, $name: expr) => {
        assert_eq!(writer_read_all($py, $module, $name), vec![]);
    };
}

pub fn reader_send_all(py: Python, module: &PyModule, name: &str, ev_list: &Vec<EvdevInputEvent>) {
    let target = module.getattr(name).unwrap().to_object(py);

    for ev in ev_list.iter() {
        let ev = serde_json::to_string(ev).unwrap();
        target.call_method(py, "__test__write_ev", (ev,), None).unwrap();
    }
}

pub fn keys(input: &str) -> Vec<EvdevInputEvent> {
    parse_key_sequence(input, Some(&Default::default())).unwrap().to_input_ev()
}

#[macro_export]
macro_rules! send {
    ($reader: expr, $keys: expr) => {
        reader_send_all(py, m, $reader, &keys($keys));
    };
}

#[macro_export]
macro_rules! assert_output {
    ($writer: expr, $keys: expr) => {
        assert_eq!(writer_read_all(py, m, $writer), keys($keys),);
    };
}

#[macro_export]
macro_rules! sleep {
    ($millis: expr) => {};
}
