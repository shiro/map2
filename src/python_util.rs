use crate::device::virtual_input_device::NativeDeviceInfo;
use crate::python::*;
use crate::*;

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
