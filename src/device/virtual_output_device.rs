use evdev_rs::{Device, UInputDevice, UninitDevice};

use crate::*;
use crate::device::virt_device::DeviceCapabilities;

use super::*;

pub struct VirtualOutputDevice {
    output_device: UInputDevice,
}

impl VirtualOutputDevice {
    pub fn send(&mut self, ev: &InputEvent) -> Result<()> {
        self.output_device.write_event(&ev)
            .map_err(|err| anyhow!("failed to write event into uinput device: {}", err))
    }
}

pub enum DeviceInitPolicy {
    NewDevice(String, DeviceCapabilities),
    CloneExistingDevice(String),
}

pub fn init_virtual_output_device(init_policy: &DeviceInitPolicy) -> Result<VirtualOutputDevice> {
    let mut new_device = UninitDevice::new()
        .ok_or(anyhow!("failed to instantiate udev device: libevdev didn't return a device"))?
        .unstable_force_init();

    match init_policy {
        DeviceInitPolicy::NewDevice(name, capabilities) => {
            virt_device::init_virtual_device(&mut new_device, name, capabilities)
                .map_err(|err| anyhow!("failed to instantiate udev device: {}", err))?;
        }
        DeviceInitPolicy::CloneExistingDevice(existing_device_fd_path) => {
            virt_device::clone_virtual_device(&mut new_device, existing_device_fd_path)
                .map_err(|err| anyhow!("failed to clone existing udev device: {}", err))?;
        }
    }

    let input_device = UInputDevice::create_from_device(&new_device);

    if let Err(err) = &input_device {
        if err.kind() == io::ErrorKind::PermissionDenied {
            return Err(anyhow!("failed to obtain write access to '/dev/uinput': {}", err));
        }
    };

    let output_device = input_device.map_err(|err| anyhow!("failed to initialize uinput device: {}", err))?;

    Ok(VirtualOutputDevice {
        output_device,
    })
}
