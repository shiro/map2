use std::collections::HashSet;
use anyhow::Result;
use evdev_rs::*;
use evdev_rs::Device;
use evdev_rs::enums::*;
use libc::clone;

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
        self.bits.insert(EventCode::EV_MSC(EV_MSC::MSC_SCAN));
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

fn clone_code_bits(src: &Device, dst: &Device, ev_code: &EventCode, max: &EventCode) -> Result<()> {
    for code in ev_code.iter() {
        if code == *max {
            break;
        }

        if !src.has(&code) {
            continue;
        }

        dst.enable(&code)
            .map_err(|err| anyhow!("failed to enable code bit: {}", err))?;
    }
    Ok(())
}

fn clone_device_props(src: &Device, mut dst: &mut Device) {
    if let Some(v) = src.name() { dst.set_name(v); }
    dst.set_vendor_id(src.vendor_id());
    if let Some(v) = src.phys() { dst.set_phys(v); }
    dst.set_bustype(src.bustype());
    dst.set_product_id(src.product_id());
    dst.set_vendor_id(src.vendor_id());
    if let Some(v) = src.uniq() { dst.set_uniq(v); }

    for prop in InputProp::INPUT_PROP_POINTER.iter() {
        if prop == InputProp::INPUT_PROP_MAX { break; }

        if src.has_property(&prop) {
            dst.enable_property(&prop).unwrap();
        }
    }
}

fn clone_device_bits(src: &Device, dst: &Device) -> Result<()> {
    for ev_type in EventType::EV_SYN.iter() {
        match ev_type {
            EventType::EV_KEY => clone_code_bits(src, dst,
                                                 &EventCode::EV_KEY(EV_KEY::KEY_RESERVED),
                                                 &EventCode::EV_KEY(EV_KEY::KEY_MAX),
            )?,
            EventType::EV_REL => clone_code_bits(src, dst,
                                                 &EventCode::EV_REL(EV_REL::REL_X),
                                                 &EventCode::EV_REL(EV_REL::REL_MAX),
            )?,
            EventType::EV_ABS => clone_code_bits(src, dst,
                                                 &EventCode::EV_ABS(EV_ABS::ABS_X),
                                                 &EventCode::EV_ABS(EV_ABS::ABS_MAX),
            )?,
            EventType::EV_LED => clone_code_bits(src, dst,
                                                 &EventCode::EV_LED(EV_LED::LED_NUML),
                                                 &EventCode::EV_LED(EV_LED::LED_MAX),
            )?,
            EventType::EV_MSC => clone_code_bits(src, dst,
                                                 &EventCode::EV_MSC(EV_MSC::MSC_SERIAL),
                                                 &EventCode::EV_MSC(EV_MSC::MSC_MAX),
            )?,
            _ => (),
        }
    }
    Ok(())
}

pub(crate) fn init_virtual_device(mut dev: &mut Device, name: &str, capabilities: &DeviceCapabilities) -> Result<()> {
    dev.set_name(name);
    enable_device_capabilities(&mut dev, &capabilities)?;

    Ok(())
}

pub(crate) fn clone_virtual_device(mut dev: &mut Device, existing_device_fd_path: &str) -> Result<()> {
    let fd_file = fs::OpenOptions::new()
        .read(true)
        .open(existing_device_fd_path)?;
    let mut device = Device::new_from_file(fd_file).unwrap();

    clone_device_props(&device, dev);
    clone_device_bits(&device, dev).unwrap();

    Ok(())
}
