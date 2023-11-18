use thiserror::Error;

use crate::*;
use crate::python::*;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("expected a callable object")]
    NotCallable
}

impl Into<PyErr> for InputError { fn into(self) -> PyErr { PyRuntimeError::new_err(self.to_string()) } }