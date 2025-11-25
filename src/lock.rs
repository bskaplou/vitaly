use crate::common;
use crate::keymap;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};
use std::collections::HashMap;
use std::{thread, time};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    unlock: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, &None)?;
    if capabilities.vial_version == 0 {
        println!("Device doesn't support locking");
    } else {
        let mut status = protocol::get_locked_status(&dev)?;
        // println!("{:?}", status);
        println!("Device is locked: {}", status.locked);
        if status.locked && unlock {
            println!("Starting unlock process... ");
            println!("Push marked buttons and keep then pushed to unlock...");
            let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"])?;
            let mut button_labels = HashMap::new();
            for (row, col) in &status.unlock_buttons {
                button_labels.insert((*row, *col), "☆☆,☆☆".to_string());
            }
            keymap::render_and_dump(&buttons, Some(button_labels));
            if !status.unlock_in_progress {
                protocol::start_unlock(&dev)?;
            }
            let second = time::Duration::from_millis(1000);
            let mut unlocked = false;
            let mut seconds_remaining: u8;
            while !unlocked {
                thread::sleep(second);
                (unlocked, seconds_remaining) = protocol::unlock_poll(&dev)?;
                println!("Seconds remaining: {} keep pushing...", seconds_remaining);
            }
            status = protocol::get_locked_status(&dev)?;
            println!("Device is locked: {}", status.locked);
        }
    }

    Ok(())
}
