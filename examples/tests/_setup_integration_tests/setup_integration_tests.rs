#[pyo3_asyncio::tokio::main]
async fn main() -> pyo3::PyResult<()> {
    std::process::Command::new("maturin")
        .arg("dev")
        .arg("--")
        .arg("--cfg")
        .arg("test")
        .output()
        .unwrap();

    pyo3_asyncio::testing::main().await
}

#[path = "../"]
mod integration_tests {
    automod::dir!("examples/tests");
}