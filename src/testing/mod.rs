use crate::*;
#[cfg(feature = "integration")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "integration")]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TestEvent {
    WriterOutEv(EvdevInputEvent),
}
