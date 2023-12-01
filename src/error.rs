use thiserror::Error;

use crate::python::*;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("expected a callable object")]
    NotCallable,
    #[error("unsupported platform, supported platforms are: Hyprland, X11")]
    UnsupportedPlatform,
    #[error("[KEY_PARSE] invalid key:\n{0}")]
    KeyParse(String),
    #[error("[KEY_SEQ_PARSE] invalid key sequence:\n{0}")]
    KeySequenceParse(String),
    #[error("[INVALID_LINK_TARGET] invalid link target")]
    InvalidLinkTarget,
}

// impl Into<PyErr> for ApplicationError { fn into(self) -> PyErr { PyRuntimeError::new_err(self.to_string()) } }

impl From<ApplicationError> for PyErr {
    fn from(value: ApplicationError) -> Self {
        PyRuntimeError::new_err(value.to_string())
    }
}

impl ApplicationError {
    pub fn into_py(self) -> PyErr { self.into() }
}