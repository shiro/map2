use evdev_rs::{UInputDevice, UninitDevice};
use crate::*;
use super::*;

pub async fn init_virtual_output_device(
    mut reader_rx: mpsc::Receiver<InputEvent>,
) -> Result<()> {
    let mut new_device = UninitDevice::new()
        .ok_or(anyhow!("failed to instantiate udev device: libevdev didn't return a device"))?
        .unstable_force_init();

    virt_device::init_virtual_device(&mut new_device)
        .map_err(|err| anyhow!("failed to instantiate udev device: {}", err))?;

    let input_device = UInputDevice::create_from_device(&new_device);

    if let Err(err) = &input_device {
        if err.kind() == io::ErrorKind::PermissionDenied {
            return Err(anyhow!("failed to obtain write access to '/dev/uinput': {}", err));
        }
    };

    let input_device = input_device.map_err(|err| anyhow!("failed to initialize uinput device: {}", err))?;

    task::spawn(async move {
        loop {
            let msg = reader_rx.recv().await;
            let ev: InputEvent = match msg {
                Some(v) => v,
                None => return Err(anyhow!("message channel closed unexpectedly")),
            };
            input_device.write_event(&ev)
                .map_err(|err| anyhow!("failed to write event into uinput device: {}", err))?;
        }
        #[allow(unreachable_code)]
            Ok(())
    });
    Ok(())
}
