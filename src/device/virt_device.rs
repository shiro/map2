use anyhow::{Result};
use evdev_rs::*;
use evdev_rs::Device;
use evdev_rs::enums::*;

fn set_code_bits(dev: &Device, ev_code: &EventCode, max: &EventCode) -> Result<()> {
    for code in ev_code.iter() {
        if code == *max {
            break;
        }

        dev.enable(&code).unwrap();
    }
    Ok(())
}

fn set_bits(dev: &Device) -> Result<()> {
    for ev_type in EventType::EV_SYN.iter() {
        match ev_type {
            EventType::EV_KEY => set_code_bits(
                dev,
                &EventCode::EV_KEY(EV_KEY::KEY_RESERVED),
                &EventCode::EV_KEY(EV_KEY::KEY_MAX),
            )?,
            EventType::EV_REL => set_code_bits(
                dev,
                &EventCode::EV_REL(EV_REL::REL_X),
                &EventCode::EV_REL(EV_REL::REL_MAX),
            )?,
            // EventType::EV_ABS => clone_code_bits(
            //     dev,
            //     &EventCode::EV_ABS(EV_ABS::ABS_X),
            //     &EventCode::EV_ABS(EV_ABS::ABS_MAX),
            // )?,
            // EventType::EV_LED => {}
                // clone_code_bits(
                // dev,
                // &EventCode::EV_LED(EV_LED::LED_NUML),
                // &EventCode::EV_LED(EV_LED::LED_MAX),
            // )?,
            _ => (),
        }
    }
    Ok(())
}

pub(crate) fn init_virtual_device(dev: &Device) -> Result<()> {
    dev.set_name("Virtual Device");
    set_bits(dev)?;

    Ok(())
}
