use std::str::FromStr;
use evdev_rs::enums::{EV_ABS, EV_REL};

use super::*;


pub fn number(input: &str) -> ParseResult<&str, i32> {
    map_res(
        recognize(tuple((opt(tag_custom("-")), digit1))),
        str::parse
    )(input)
}

pub fn motion_action(input: &str) -> ParseResult<&str, KeyAction> {
    map_res(
        tuple((
            alt((
                tuple((ident, multispace1, ident)),
            )),
            multispace1,
            number,
        )),
        |((tag1, _, tag2), _, value)| {
            let event_code = match &*tag1 {
                "relative" => {
                    match &*tag2 {
                        "X" => EventCode::EV_REL(EV_REL::REL_X),
                        "Y" => EventCode::EV_REL(EV_REL::REL_Y),
                        "Z" => EventCode::EV_REL(EV_REL::REL_Z),
                        "RX" => EventCode::EV_REL(EV_REL::REL_RX),
                        "RY" => EventCode::EV_REL(EV_REL::REL_RY),
                        "RZ" => EventCode::EV_REL(EV_REL::REL_RZ),
                        "HWHEEL" => EventCode::EV_REL(EV_REL::REL_HWHEEL),
                        "DIAL" => EventCode::EV_REL(EV_REL::REL_DIAL),
                        "WHEEL" => EventCode::EV_REL(EV_REL::REL_WHEEL),
                        "MISC" => EventCode::EV_REL(EV_REL::REL_MISC),
                        "RESERVED" => EventCode::EV_REL(EV_REL::REL_RESERVED),
                        "WHEEL_HI_RES" => EventCode::EV_REL(EV_REL::REL_WHEEL_HI_RES),
                        "HWHEEL_HI_RES" => EventCode::EV_REL(EV_REL::REL_HWHEEL_HI_RES),
                        "MAX" => EventCode::EV_REL(EV_REL::REL_MAX),
                        _ => return Err(make_generic_nom_err_new(input))
                    }
                }
                "absolute" => {
                    match &*tag2 {
                        "X" => EventCode::EV_ABS(EV_ABS::ABS_X),
                        "Y" => EventCode::EV_ABS(EV_ABS::ABS_Y),
                        "Z" => EventCode::EV_ABS(EV_ABS::ABS_Z),
                        "RX" => EventCode::EV_ABS(EV_ABS::ABS_RX),
                        "RY" => EventCode::EV_ABS(EV_ABS::ABS_RY),
                        "RZ" => EventCode::EV_ABS(EV_ABS::ABS_RZ),
                        "THROTTLE" => EventCode::EV_ABS(EV_ABS::ABS_THROTTLE),
                        "RUDDER" => EventCode::EV_ABS(EV_ABS::ABS_RUDDER),
                        "WHEEL" => EventCode::EV_ABS(EV_ABS::ABS_WHEEL),
                        "GAS" => EventCode::EV_ABS(EV_ABS::ABS_GAS),
                        "BRAKE" => EventCode::EV_ABS(EV_ABS::ABS_BRAKE),
                        "HAT0X" => EventCode::EV_ABS(EV_ABS::ABS_HAT0X),
                        "HAT0Y" => EventCode::EV_ABS(EV_ABS::ABS_HAT0Y),
                        "HAT1X" => EventCode::EV_ABS(EV_ABS::ABS_HAT1X),
                        "HAT1Y" => EventCode::EV_ABS(EV_ABS::ABS_HAT1Y),
                        "HAT2X" => EventCode::EV_ABS(EV_ABS::ABS_HAT2X),
                        "HAT2Y" => EventCode::EV_ABS(EV_ABS::ABS_HAT2Y),
                        "HAT3X" => EventCode::EV_ABS(EV_ABS::ABS_HAT3X),
                        "HAT3Y" => EventCode::EV_ABS(EV_ABS::ABS_HAT3Y),
                        "PRESSURE" => EventCode::EV_ABS(EV_ABS::ABS_PRESSURE),
                        "DISTANCE" => EventCode::EV_ABS(EV_ABS::ABS_DISTANCE),
                        "TILT_X" => EventCode::EV_ABS(EV_ABS::ABS_TILT_X),
                        "TILT_Y" => EventCode::EV_ABS(EV_ABS::ABS_TILT_Y),
                        "TOOL_WIDTH" => EventCode::EV_ABS(EV_ABS::ABS_TOOL_WIDTH),
                        "VOLUME" => EventCode::EV_ABS(EV_ABS::ABS_VOLUME),
                        "MISC" => EventCode::EV_ABS(EV_ABS::ABS_MISC),
                        "RESERVED" => EventCode::EV_ABS(EV_ABS::ABS_RESERVED),
                        "MT_SLOT" => EventCode::EV_ABS(EV_ABS::ABS_MT_SLOT),
                        "MT_TOUCH_MAJOR" => EventCode::EV_ABS(EV_ABS::ABS_MT_TOUCH_MAJOR),
                        "MT_TOUCH_MINOR" => EventCode::EV_ABS(EV_ABS::ABS_MT_TOUCH_MINOR),
                        "MT_WIDTH_MAJOR" => EventCode::EV_ABS(EV_ABS::ABS_MT_WIDTH_MAJOR),
                        "MT_WIDTH_MINOR" => EventCode::EV_ABS(EV_ABS::ABS_MT_WIDTH_MINOR),
                        "MT_ORIENTATION" => EventCode::EV_ABS(EV_ABS::ABS_MT_ORIENTATION),
                        "MT_POSITION_X" => EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_X),
                        "MT_POSITION_Y" => EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_Y),
                        "MT_TOOL_TYPE" => EventCode::EV_ABS(EV_ABS::ABS_MT_TOOL_TYPE),
                        "MT_BLOB_ID" => EventCode::EV_ABS(EV_ABS::ABS_MT_BLOB_ID),
                        "MT_TRACKING_ID" => EventCode::EV_ABS(EV_ABS::ABS_MT_TRACKING_ID),
                        "MT_PRESSURE" => EventCode::EV_ABS(EV_ABS::ABS_MT_PRESSURE),
                        "MT_DISTANCE" => EventCode::EV_ABS(EV_ABS::ABS_MT_DISTANCE),
                        "MT_TOOL_X" => EventCode::EV_ABS(EV_ABS::ABS_MT_TOOL_X),
                        "MT_TOOL_Y" => EventCode::EV_ABS(EV_ABS::ABS_MT_TOOL_Y),
                        "MAX" => EventCode::EV_ABS(EV_ABS::ABS_MAX),
                        _ => return Err(make_generic_nom_err_new(input))
                    }
                }
                _ => return Err(make_generic_nom_err_new(input))
            };


            Ok::<_, nom::Err<CustomError<_>>>(KeyAction::new(Key { event_code }, value as i32))
        },
    )(input)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_action_input() {
        assert_eq!(motion_action("relative X 33"), nom_ok(
            KeyAction { key: Key { event_code: EventCode::EV_REL(EV_REL::REL_X) }, value: 33 }
        ));
        assert_eq!(motion_action("relative Y 99"), nom_ok(
            KeyAction { key: Key { event_code: EventCode::EV_REL(EV_REL::REL_Y) }, value: 99 }
        ));

        assert_eq!(motion_action("absolute Z 99"), nom_ok(
            KeyAction { key: Key { event_code: EventCode::EV_ABS(EV_ABS::ABS_Z) }, value: 99 }
        ));

        assert_eq!(motion_action("absolute TILT_X -5"), nom_ok(
            KeyAction { key: Key { event_code: EventCode::EV_ABS(EV_ABS::ABS_TILT_X) }, value: -5 }
        ));
    }

    #[test]
    fn motion_action_invalid_input() {
        assert_nom_err(motion_action("relative foo 33"), "relative foo 33");
    }
}