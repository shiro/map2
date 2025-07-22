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
    #[error("[INVALID_INPUT_TYPE] expected input '{name}' to be of type {expected_type}, not {actual_type}")]
    InvalidInputType { name: String, expected_type: String, actual_type: String },
    #[error(
        "[INVALID_NAMED_INPUT_TYPE] expected named input '{name}' to be of type {expected_type}, not {actual_type}"
    )]
    InvalidNamedInputType { name: String, expected_type: String, actual_type: String },
    #[error("[UNEXPECTED_NON_BUTTON_INPUT] expected only button inputs")]
    NonButton,
    #[error("[TOO_MANY_EVENTS] can't keep up with event processing, dropping events!")]
    TooManyEvents,
}

impl From<ApplicationError> for PyErr {
    fn from(value: ApplicationError) -> Self {
        match value {
            ApplicationError::InvalidInputType { .. } | ApplicationError::InvalidNamedInputType { .. } => {
                PyTypeError::new_err(value.to_string())
            }
            _ => PyRuntimeError::new_err(value.to_string()),
        }
    }
}

impl ApplicationError {
    pub fn into_py(self) -> PyErr {
        self.into()
    }
}
