use std::collections::HashSet;

use anyhow::Result;
use evdev_rs::enums::*;
use evdev_rs::Device;
use evdev_rs::*;

use crate::*;

#[derive(Default)]
pub struct DeviceCapabilities {
    abs_bits: HashSet<(EventCode, AbsInfo)>,
    bits: HashSet<EventCode>,
}

impl DeviceCapabilities {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    fn set_bit_range(&mut self, min: &EventCode, max: &EventCode) {
        for code in min.iter() {
            if code == *max {
                break;
            }
            self.bits.insert(code);
        }
    }

    pub fn enable_all_keyboard(&mut self) {
        for bit in ALL_KEYS {
            self.bits.insert(EventCode::EV_KEY(*bit));
        }
        self.bits.insert(EventCode::EV_MSC(EV_MSC::MSC_SCAN));
    }
    pub fn enable_all_buttons(&mut self) {
        for bit in ALL_BUTTONS {
            self.bits.insert(EventCode::EV_KEY(*bit));
        }
    }
    pub fn enable_all_rel(&mut self) {
        for bit in ALL_REL {
            self.bits.insert(EventCode::EV_REL(*bit));
        }
    }
    pub fn enable_all_abs(&mut self) {
        for bit in ALL_ABS {
            self.abs_bits.insert((
                EventCode::EV_ABS(*bit),
                AbsInfo { value: 128, minimum: 0, maximum: 255, fuzz: 0, flat: 0, resolution: 0 },
            ));
        }
    }
    pub fn enable_abs(&mut self, code: EV_ABS, info: AbsInfo) {
        self.abs_bits.insert((EventCode::EV_ABS(code), info));
    }
}

pub fn enable_device_capabilities(dev: &mut Device, capabilities: &DeviceCapabilities) -> Result<()> {
    for (code, abs_info) in capabilities.abs_bits.iter() {
        dev.enable_event_code(code, Some(abs_info))
            .map_err(|err| anyhow!("failed to enable code bit '{}': {}", code, err))?;
    }

    for code in capabilities.bits.iter() {
        dev.enable(code).map_err(|err| anyhow!("failed to enable code bit '{}': {}", code, err))?;
    }

    Ok(())
}

fn set_code_bits(dev: &Device, ev_code: &EventCode, max: &EventCode) -> Result<()> {
    for code in ev_code.iter() {
        if code == *max {
            break;
        }

        dev.enable(&code).map_err(|err| anyhow!("failed to enable code bit: {}", err))?;
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

        dst.enable(&code).map_err(|err| anyhow!("failed to enable code bit: {}", err))?;
    }
    Ok(())
}

fn clone_device_props(src: &Device, dst: &mut Device) {
    if let Some(v) = src.name() {
        dst.set_name(v);
    }
    dst.set_vendor_id(src.vendor_id());
    if let Some(v) = src.phys() {
        dst.set_phys(v);
    }
    dst.set_bustype(src.bustype());
    dst.set_product_id(src.product_id());
    dst.set_vendor_id(src.vendor_id());
    if let Some(v) = src.uniq() {
        dst.set_uniq(v);
    }

    for prop in InputProp::INPUT_PROP_POINTER.iter() {
        if prop == InputProp::INPUT_PROP_MAX {
            break;
        }

        if src.has_property(&prop) {
            dst.enable_property(&prop).unwrap();
        }
    }
}

fn clone_device_bits(src: &Device, dst: &Device) -> Result<()> {
    for ev_type in EventType::EV_SYN.iter() {
        match ev_type {
            EventType::EV_KEY => clone_code_bits(
                src,
                dst,
                &EventCode::EV_KEY(EV_KEY::KEY_RESERVED),
                &EventCode::EV_KEY(EV_KEY::KEY_MAX),
            )?,
            EventType::EV_REL => clone_code_bits(src, dst, &EventCode::EV_REL(REL_X), &EventCode::EV_REL(REL_MAX))?,
            EventType::EV_ABS => clone_code_bits(src, dst, &EventCode::EV_ABS(ABS_X), &EventCode::EV_ABS(ABS_MAX))?,
            EventType::EV_LED => {
                clone_code_bits(src, dst, &EventCode::EV_LED(EV_LED::LED_NUML), &EventCode::EV_LED(EV_LED::LED_MAX))?
            }
            EventType::EV_MSC => {
                clone_code_bits(src, dst, &EventCode::EV_MSC(EV_MSC::MSC_SERIAL), &EventCode::EV_MSC(EV_MSC::MSC_MAX))?
            }
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

pub(crate) fn clone_virtual_device(dev: &mut Device, existing_device_fd_path: &str) -> Result<()> {
    let fd_file = fs::OpenOptions::new().read(true).open(existing_device_fd_path)?;
    let device = Device::new_from_file(fd_file).unwrap();

    clone_device_props(&device, dev);
    clone_device_bits(&device, dev).unwrap();

    Ok(())
}
