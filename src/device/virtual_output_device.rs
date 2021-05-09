use crate::*;
use super::*;
use evdev_rs::{UninitDevice, UInputDevice};

pub async fn init_virtual_output_device(
    mut reader_rx: mpsc::Receiver<InputEvent>,
) -> Result<()> {
    let mut new_device = UninitDevice::new()
        .ok_or(anyhow!("failed to instantiate udev device"))?
        .unstable_force_init();
    virt_device::init_virtual_device(&mut new_device).unwrap();

    let input_device = UInputDevice::create_from_device(&new_device)?;

    task::spawn(async move {
        loop {
            let msg = reader_rx.recv().await;
            let ev: InputEvent = match msg {
                Some(v) => v,
                None => return Err(anyhow!("message channel closed unexpectedly")),
            };
            input_device.write_event(&ev)?;
        }
        Ok(())
    });
    Ok(())
}
