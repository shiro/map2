use crate::device::virtual_input_device::NativeDeviceInfo;
use crate::python::*;

pub fn device_info_to_py(py: Python, device_info: &NativeDeviceInfo) -> PyObject {
    let py_dict = PyDict::new(py);
    py_dict.set_item("path", PyString::new(py, &device_info.fd_path.to_string_lossy().to_string())).unwrap();
    py_dict.set_item("sys_path", PyString::new(py, &device_info.sys_path.to_string_lossy().to_string())).unwrap();

    let properties_dict = PyDict::new(py);
    for (key, value) in &device_info.properties {
        properties_dict.set_item(PyString::new(py, key), PyString::new(py, value)).unwrap();
    }
    py_dict.set_item("properties", properties_dict).unwrap();

    py_dict.into()
}

pub struct PyRegex {
    inner: PyObject,
}

impl PyRegex {
    pub fn is_match(&self, py: Python, needle: &str) -> bool {
        match self.inner.call_method(py, "search", (needle,), None) {
            Ok(v) => !v.is_none(py),
            Err(_) => false,
        }
    }
}

impl<'source> FromPyObject<'source> for PyRegex {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if ob.get_type().str().is_ok_and(|v| v == "<class 're.Pattern'>") {
            Ok(PyRegex { inner: ob.clone().unbind() })
        } else {
            Err(PyTypeError::new_err("Expected a Python 're.Pattern' object"))
        }
    }
}
