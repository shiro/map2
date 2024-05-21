use serde::*;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsInfo {
    #[serde(default)]
    pub value: i32,
    #[serde(default)]
    #[serde(rename(deserialize = "min"))]
    pub minimum: i32,
    #[serde(default)]
    #[serde(rename(deserialize = "max"))]
    pub maximum: i32,
    #[serde(default)]
    pub fuzz: i32,
    #[serde(default)]
    pub flat: i32,
    #[serde(default)]
    pub resolution: i32,
}

impl AbsInfo {
    pub fn into_evdev(self) -> evdev_rs::AbsInfo {
        evdev_rs::AbsInfo {
            value: self.value,
            minimum: self.minimum,
            maximum: self.maximum,
            fuzz: self.fuzz,
            flat: self.flat,
            resolution: self.resolution,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbsSpec {
    Bool(bool),
    AbsInfo(AbsInfo),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Abs {
    Bool(bool),
    Specification(HashMap<String, AbsSpec>),
}

impl Default for Abs {
    fn default() -> Self {
        Self::Bool(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub rel: bool,
    #[serde(default)]
    pub abs: Abs,
    #[serde(default)]
    pub keys: bool,
    #[serde(default)]
    pub buttons: bool,
}
