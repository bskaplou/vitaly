use crate::common;
use crate::keymap;
use crate::protocol;
use anyhow::{Result, anyhow};
use hidapi::{DeviceInfo, HidApi};
use std::{thread, time};

pub fn run(api: &HidApi, device: &DeviceInfo, meta_file: &Option<String>) -> Result<()> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;

    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or(anyhow!("matrix/cols not found in meta"))? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or(anyhow!("matrix/rows not found in meta"))? as u8;

    let state = protocol::load_layout_options(&dev)?;
    let options = protocol::LayoutOptions::from_json(state, &meta["layouts"]["labels"])?;
    let mut buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"], &options)?;

    let sleep_duration = time::Duration::from_millis(50);
    let mut last_state: Option<protocol::MatrixState> = None;
    for button in &mut buttons {
        button.color = None;
    }
    if capabilities.vial_version > 0 {
        let status = protocol::get_locked_status(&dev)?;
        if status.locked {
            return Err(common::CommandError("Keyboard is locked, it's necessary to unlock it to run tester, keyboard might be unlocked with subcommand 'lock -u'".to_string()).into());
        }
    }
    println!("Tap buttons. Press Ctrl+c to finish.");
    keymap::render_and_dump(&buttons, None);

    loop {
        let state = protocol::matrix_poll(&dev, rows, cols)?;
        if let Some(last_state) = last_state
            && state != last_state
        {
            for button in &mut buttons {
                if !button.encoder {
                    if state.is_pushed(button.wire_x, button.wire_y, capabilities.via_version)? {
                        button.color = Some((255, 255, 255));
                    } else if button.color.is_some() {
                        button.color = Some((150, 150, 150));
                    }
                }
            }
            println!();
            keymap::render_and_dump(&buttons, None);
        }
        last_state = Some(state);
        thread::sleep(sleep_duration);
    }
}
