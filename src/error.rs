use thiserror::Error;

use crate::python::*;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("expected a callable object")]
    NotCallable,
    #[error("unsupported platform, supported platforms are: Hyprland")]
    UnsupportedPlatform,
}

impl Into<PyErr> for ApplicationError { fn into(self) -> PyErr { PyRuntimeError::new_err(self.to_string()) } }