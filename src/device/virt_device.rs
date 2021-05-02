use anyhow::{Result};
use evdev_rs::*;
use evdev_rs::Device;
use evdev_rs::enums::*;
use walkdir::{DirEntry, WalkDir};

fn clone_code_bits(dev: &Device, ev_code: &EventCode, max: &EventCode) -> Result<()> {
    for code in ev_code.iter() {
        if code == *max {
            break;
        }

        dev.enable(&code).unwrap();
        // println!("    Event code: {}", code);
        // match code {
        //     EventCode::EV_ABS(k) => print_abs_bits(src, &k),
        //     // _ => (),
        // }
    }
    Ok(())
}

fn clone_bits(dev: &Device) -> Result<()> {
    // println!("Supported events:");
    for ev_type in EventType::EV_SYN.iter() {
        match ev_type {
            EventType::EV_KEY => clone_code_bits(
                dev,
                &EventCode::EV_KEY(EV_KEY::KEY_RESERVED),
                &EventCode::EV_KEY(EV_KEY::KEY_MAX),
            )?,
            EventType::EV_REL => clone_code_bits(
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

fn clone_props(dev: &Device) -> Result<()> {
    for input_prop in InputProp::INPUT_PROP_POINTER.iter() {
        dev.enable(&input_prop)?;
    }
    Ok(())
}

pub(crate) fn setup_virt_device(dev: &Device) -> Result<()> {
    dev.set_name("Virtual Device");

    // clone_props(dev)?;
    clone_bits(dev)?;

    // dst.set_phys(phys);
    // dst.set_uniq(uniq);


    Ok(())
}
