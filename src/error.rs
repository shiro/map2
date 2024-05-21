use thiserror::Error;

use crate::python::*;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("[UNSUPPORTED_PLATFORM] unsupported platform, supported platforms are: Hyprland, X11")]
    UnsupportedPlatform,
    #[error("[KEY_PARSE] invalid key:\n{0}")]
    KeyParse(String),
    #[error("[KEY_SEQ_PARSE] invalid key sequence:\n{0}")]
    KeySequenceParse(String),
    #[error("[INVALID_LINK_TARGET] invalid link target")]
    InvalidLinkTarget,
    #[error("[NOT_CALLABLE] expected a callable object (i.e. a function)")]
    NotCallable,
    #[error("[INVALID_INPUT_TYPE] expected input to be of type {type_}")]
    InvalidInputType { type_: String },
    #[error("[UNEXPECTED_NON_BUTTON_INPUT] expected only button inputs")]
    NonButton,
}

impl From<ApplicationError> for PyErr {
    fn from(value: ApplicationError) -> Self {
        PyRuntimeError::new_err(value.to_string())
    }
}

impl ApplicationError {
    pub fn into_py(self) -> PyErr {
        self.into()
    }
}

