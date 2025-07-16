use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = map2::python::doc_stub_info()?;
    stub.generate()?;
    Ok(())
}
