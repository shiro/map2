use std::collections::HashSet;
use anyhow::Result;
use evdev_rs::*;
use evdev_rs::Device;
use evdev_rs::enums::*;

use crate::*;

pub struct DeviceCapabilities {
    bits: HashSet<EventCode>,
}

impl DeviceCapabilities {
    pub(crate) fn new() -> Self {
        Self { bits: HashSet::new() }
    }

    fn set_bit_range(&mut self, min: &EventCode, max: &EventCode) {
        for code in min.iter() {
            if code == *max {
                break;
            }
            self.bits.insert(code);
        }
    }

    pub fn enable_keyboard(&mut self) {
        self.set_bit_range(&EventCode::EV_KEY(EV_KEY::KEY_RESERVED), &EventCode::EV_KEY(EV_KEY::KEY_MAX));
    }
    pub fn enable_buttons(&mut self) {
        self.set_bit_range(&EventCode::EV_KEY(EV_KEY::BTN_0), &EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY40));
    }
    pub fn enable_rel(&mut self) {
        self.set_bit_range(&EventCode::EV_REL(EV_REL::REL_X), &EventCode::EV_REL(EV_REL::REL_MAX));
    }
    pub fn enable_abs(&mut self) {
        self.set_bit_range(&EventCode::EV_ABS(EV_ABS::ABS_X), &EventCode::EV_ABS(EV_ABS::ABS_MAX));
    }
}

pub fn enable_device_capabilities(dev: &mut Device, capabilities: &DeviceCapabilities) -> Result<()> {
    for code in capabilities.bits.iter() {
        dev.enable(code)
            .map_err(|err| anyhow!("failed to enable code bit: {}", err))?;
    }

    Ok(())
}

fn set_code_bits(dev: &Device, ev_code: &EventCode, max: &EventCode) -> Result<()> {
    for code in ev_code.iter() {
        if code == *max {
            break;
        }

        dev.enable(&code)
            .map_err(|err| anyhow!("failed to enable code bit: {}", err))?;
    }
    Ok(())
}

pub(crate) fn init_virtual_device(mut dev: &mut Device, name: &str, capabilities: &DeviceCapabilities) -> Result<()> {
    dev.set_name(name);

    enable_device_capabilities(&mut dev, &capabilities)?;

    Ok(())
}
