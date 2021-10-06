use evdev_rs::{UInputDevice, UninitDevice};
use crate::*;
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

pub fn init_virtual_output_device(device_name: &str) -> Result<VirtualOutputDevice> {
    let mut new_device = UninitDevice::new()
        .ok_or(anyhow!("failed to instantiate udev device: libevdev didn't return a device"))?
        .unstable_force_init();

    virt_device::init_virtual_device(&mut new_device, device_name)
        .map_err(|err| anyhow!("failed to instantiate udev device: {}", err))?;

    let input_device = UInputDevice::create_from_device(&new_device);

    if let Err(err) = &input_device {
        if err.kind() == io::ErrorKind::PermissionDenied {
            return Err(anyhow!("failed to obtain write access to '/dev/uinput': {}", err));
        }
    };

    let output_device = input_device.map_err(|err| anyhow!("failed to initialize uinput device: {}", err))?;

    Ok(VirtualOutputDevice {
        output_device
    })
}
