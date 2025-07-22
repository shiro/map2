use crate::python::*;
use crate::*;

/// A trait to provide type name strings for error messages
pub trait TypeName {
    fn type_name() -> &'static str;
}

impl TypeName for String {
    fn type_name() -> &'static str {
        "str"
    }
}

impl TypeName for i32 {
    fn type_name() -> &'static str {
        "int"
    }
}

impl TypeName for i64 {
    fn type_name() -> &'static str {
        "int"
    }
}

impl TypeName for f32 {
    fn type_name() -> &'static str {
        "float"
    }
}

impl TypeName for f64 {
    fn type_name() -> &'static str {
        "float"
    }
}

impl TypeName for bool {
    fn type_name() -> &'static str {
        "bool"
    }
}

impl<T> TypeName for Vec<T> {
    fn type_name() -> &'static str {
        "list"
    }
}

impl<K, V> TypeName for std::collections::HashMap<K, V> {
    fn type_name() -> &'static str {
        "dict"
    }
}

pub fn get_py_type(value: &Bound<PyAny>) -> String {
    value.get_type().name().map_or("unknown".to_string(), |s| s.to_string())
}

/// Generic function to extract a value from a HashMap with improved error messages
pub fn extract_with_error<T>(options: &HashMap<String, Bound<PyAny>>, key: &str) -> PyResult<Option<T>>
where
    T: for<'py> FromPyObject<'py> + TypeName,
{
    match options.get(key) {
        Some(value) => {
            let extracted = value.extract::<T>().map_err(|_err| ApplicationError::InvalidNamedInputType {
                name: key.to_string(),
                actual_type: get_py_type(value),
                expected_type: T::type_name().to_string(),
            })?;
            Ok(Some(extracted))
        }
        None => Ok(None),
    }
}
