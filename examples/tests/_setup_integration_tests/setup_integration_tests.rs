use std::io::Write;

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