pub use pyo3::exceptions::PyRuntimeError;
pub use pyo3::impl_::wrap::OkWrap;
pub use pyo3::prelude::*;
pub use pyo3::types::PyDict;
use signal_hook::{consts::SIGINT, iterator::Signals};

use crate::*;
use crate::mapper::mapper::KeyMapperSnapshot;
use crate::text_mapper::TextMapper;
use crate::virtual_reader::VirtualReader;
use crate::virtual_writer::VirtualWriter;
use crate::window::Window;

#[pyclass]
struct PyKey {
    #[pyo3(get, set)]
    code: u32,
    #[pyo3(get, set)]
    value: i32,
}


#[pyfunction]
fn link(py: Python, mut chain: Vec<PyObject>) -> PyResult<()> {
    let mut prev: Option<PyObject> = None;

    for target in chain.into_iter() {
        if let Some(source) = prev {
            if let Ok(mut source) = source.extract::<PyRefMut<Reader>>(py) { source.link(target.as_ref(py))?; }
            if let Ok(mut source) = source.extract::<PyRefMut<Mapper>>(py) { source.link(target.as_ref(py))?; }
        }
        prev = Some(target);
    }

    Ok(())
}

pub fn err_to_py(err: anyhow::Error) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

#[pyfunction]
fn wait(py: Python) {
    py.allow_threads(|| {
        let mut signals = Signals::new(&[SIGINT]).unwrap();
        for _ in signals.forever() {
            std::process::exit(0);
        }
    });
}

#[pyfunction]
fn exit(exit_code: Option<i32>) { std::process::exit(exit_code.unwrap_or(0)); }

#[pymodule]
fn map2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wait, m)?)?;
    m.add_function(wrap_pyfunction!(exit, m)?)?;
    m.add_function(wrap_pyfunction!(link, m)?)?;
    m.add_class::<Reader>()?;
    m.add_class::<Mapper>()?;
    m.add_class::<KeyMapperSnapshot>()?;
    m.add_class::<TextMapper>()?;
    m.add_class::<Writer>()?;
    m.add_class::<VirtualWriter>()?;
    m.add_class::<VirtualReader>()?;
    m.add_class::<Window>()?;

    Ok(())
}
