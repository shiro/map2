use serde::*;

use crate::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsInfo {
    #[serde(default)]
    pub value: i32,
    #[serde(default)]
    pub minimum: i32,
    #[serde(default)]
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

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub enum AbsSpec {
//     X(AbsSpecInner),
//     Y(AbsSpecInner),
//     Z(AbsSpecInner),
//     RX(AbsSpecInner),
//     RY(AbsSpecInner),
//     RZ(AbsSpecInner),
//     THROTTLE(AbsSpecInner),
//     RUDDER(AbsSpecInner),
//     WHEEL(AbsSpecInner),
//     GAS(AbsSpecInner),
//     BRAKE(AbsSpecInner),
//     HAT0X(AbsSpecInner),
//     HAT0Y(AbsSpecInner),
//     HAT1X(AbsSpecInner),
//     HAT1Y(AbsSpecInner),
//     HAT2X(AbsSpecInner),
//     HAT2Y(AbsSpecInner),
//     HAT3X(AbsSpecInner),
//     HAT3Y(AbsSpecInner),
//     PRESSURE(AbsSpecInner),
//     DISTANCE(AbsSpecInner),
//     TILT_X(AbsSpecInner),
//     TILT_Y(AbsSpecInner),
//     TOOL_WIDTH(AbsSpecInner),
//     VOLUME(AbsSpecInner),
//     MISC(AbsSpecInner),
//     RESERVED(AbsSpecInner),
//     MT_SLOT(AbsSpecInner),
//     MT_TOUCH_MAJOR(AbsSpecInner),
//     MT_TOUCH_MINOR(AbsSpecInner),
//     MT_WIDTH_MAJOR(AbsSpecInner),
//     MT_WIDTH_MINOR(AbsSpecInner),
//     MT_ORIENTATION(AbsSpecInner),
//     MT_POSITION_X(AbsSpecInner),
//     MT_POSITION_Y(AbsSpecInner),
//     MT_TOOL_TYPE(AbsSpecInner),
//     MT_BLOB_ID(AbsSpecInner),
//     MT_TRACKING_ID(AbsSpecInner),
//     MT_PRESSURE(AbsSpecInner),
//     MT_DISTANCE(AbsSpecInner),
//     MT_TOOL_X(AbsSpecInner),
//     MT_TOOL_Y(AbsSpecInner),
//     MAX(AbsSpecInner),
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Abs {
    Bool(bool),
    Specification(HashMap<String, AbsSpec>),
}

impl Default for Abs {
    fn default() -> Self { Self::Bool(false) }
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