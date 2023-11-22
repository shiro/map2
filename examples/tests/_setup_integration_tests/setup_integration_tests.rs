use std::io::Write;

use map2::*;
use map2::python::*;

#[pyo3_asyncio::tokio::main]
async fn main() -> pyo3::PyResult<()> {
    let cmd = std::process::Command::new("maturin")
        .arg("dev")
        // .arg("--")
        // .arg("--cfg").arg("test")
        // .arg("--cfg").arg("integration")
        .arg("--features").arg("integration")
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

    target.call_method0(py, "try_recv").unwrap()
        .extract::<Option<String>>(py).unwrap()
        .and_then(|x| serde_json::from_str(&x).unwrap())
}

pub fn writer_read_all(py: Python, module: &PyModule, name: &str) -> Vec<EvdevInputEvent> {
    let mut acc = vec![];
    while let Some(ev) = writer_read(py, module, name){
        acc.push(ev);
    }
    acc
}

pub fn reader_send(py: Python, module: &PyModule, name: &str, ev: &EvdevInputEvent) {
    let target = module.getattr(name).unwrap().to_object(py);
    let ev = serde_json::to_string(ev).unwrap();

    target.call_method(py, "send", (ev, ), None).unwrap();
}