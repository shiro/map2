pub use pyo3::exceptions::PyRuntimeError;
pub use pyo3::impl_::wrap::OkWrap;
pub use pyo3::prelude::*;
pub use pyo3::PyClass;
pub use pyo3::types::PyDict;
use signal_hook::{consts::SIGINT, iterator::Signals};
use tokio::runtime::Runtime;

use crate::*;
use crate::mapper::mapper::KeyMapperSnapshot;
use crate::text_mapper::TextMapper;
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
#[pyo3(signature = (* * options))]
fn default(options: Option<&PyDict>) -> PyResult<()> {
    let options: HashMap<&str, &PyAny> = match options {
        Some(py_dict) => py_dict.extract().unwrap(),
        None => HashMap::new()
    };

    let kbd_model: Option<String> = options.get("model").and_then(|x| x.extract().ok());
    let kbd_layout: Option<String> = options.get("layout").and_then(|x| x.extract().ok());
    let kbd_variant: Option<Option<String>> = options.get("variant").and_then(|x| x.extract().ok());
    let kbd_options: Option<Option<String>> = options.get("options").and_then(|x| x.extract().ok());

    if kbd_model.is_some() || kbd_layout.is_some() || kbd_variant.is_some() || kbd_options.is_some() {
        let mut default_params = global::DEFAULT_TRANSFORMER_PARAMS.write().unwrap();

        if let Some(model) = kbd_model { default_params.model = model; }
        if let Some(layout) = kbd_layout { default_params.layout = layout; }
        if let Some(variant) = kbd_variant { default_params.variant = variant; }
        if let Some(options) = kbd_options { default_params.options = options; }
    }
    Ok(())
}

#[pyfunction]
fn link(py: Python, chain: Vec<PyObject>) -> PyResult<()> {
    let mut prev: Option<PyObject> = None;
    let mut path = vec![];

    for target in chain.into_iter() {
        if let Some(source) = prev {
            if let Ok(mut source) = source.extract::<PyRefMut<Reader>>(py) {
                source.link(target.as_ref(py))?;
                path.push(source.id.clone());
            }
            if let Ok(mut source) = source.extract::<PyRefMut<Mapper>>(py) {
                source.link(path.clone(), target.as_ref(py))?;
                path.push(source.id.clone());
            }
        }
        prev = Some(target);
    }

    Ok(())
}

pub fn err_to_py(err: anyhow::Error) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

pub fn get_runtime<'a>() -> &'a Runtime { pyo3_asyncio::tokio::get_runtime() }

#[pyfunction]
fn wait(py: Python) {
    #[cfg(not(feature = "integration"))]
    py.allow_threads(|| {
        let mut signals = Signals::new(&[SIGINT]).unwrap();
        for _ in signals.forever() {
            std::process::exit(0);
        }
    });
}

#[pyfunction]
fn exit(exit_code: Option<i32>) {
    #[cfg(not(feature = "integration"))]
    std::process::exit(exit_code.unwrap_or(0));
}

#[cfg(feature = "integration")]
#[pyfunction]
fn __test() -> PyResult<Vec<String>> {
    Ok(global::TEST_PIPE.lock().unwrap()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap())
        .collect()
    )
}

#[pymodule]
fn map2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wait, m)?)?;
    m.add_function(wrap_pyfunction!(exit, m)?)?;
    m.add_function(wrap_pyfunction!(default, m)?)?;
    m.add_function(wrap_pyfunction!(link, m)?)?;
    #[cfg(feature = "integration")]
    m.add_function(wrap_pyfunction!(__test, m)?)?;
    m.add_class::<Reader>()?;
    m.add_class::<Mapper>()?;
    m.add_class::<KeyMapperSnapshot>()?;
    m.add_class::<TextMapper>()?;
    m.add_class::<Writer>()?;
    m.add_class::<VirtualWriter>()?;
    m.add_class::<Window>()?;

    Ok(())
}
