use crate::keymap;
use crate::protocol;
use hidapi::HidDevice;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
#[error("{0}")]
pub struct CommandError(pub String);

pub fn load_meta(
    dev: &HidDevice,
    capabilities: &protocol::Capabilities,
    meta_file: &Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match meta_file {
        Some(meta_file) => {
            let meta_str = fs::read_to_string(meta_file)?;
            Ok(serde_json::from_str(&meta_str)?)
        }
        None => {
            if capabilities.vial_version == 0 {
                return Err(CommandError(
                    "device doesn't support vial protocol"
                        .to_string()
                        .to_string(),
                )
                .into());
            }
            let meta_data = match protocol::load_vial_meta(dev) {
                Ok(meta_data) => meta_data,
                Err(e) => {
                    return Err(CommandError(
                        format!("failed to load vial meta {:?}", e).to_string(),
                    )
                    .into());
                }
            };
            Ok(meta_data)
        }
    }
}

pub fn render_layer(
    keys: &protocol::Keymap,
    buttons: &Vec<keymap::Button>,
    layer_number: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut button_labels = HashMap::new();
    let mut fat_labels = Vec::new();
    for button in buttons {
        let label = keys.get_short(layer_number, button.wire_x, button.wire_y)?;
        let mut slim_label = true;
        for (idx, part) in label.split(',').enumerate() {
            if part.chars().count() > 3 || idx > 1 {
                slim_label &= false;
            }
        }
        if !slim_label {
            match fat_labels.iter().position(|e| *e == label) {
                None => {
                    fat_labels.push(label);
                    button_labels.insert(
                        (button.wire_x, button.wire_y),
                        format!("*{}", fat_labels.len()),
                    );
                }
                Some(pos) => {
                    button_labels.insert((button.wire_x, button.wire_y), format!("*{}", pos));
                }
            }
        } else {
            button_labels.insert((button.wire_x, button.wire_y), label);
        }
    }
    println!("Layer: {}", layer_number);
    keymap::render_and_dump(buttons, Some(button_labels));
    for (idx, fat) in fat_labels.into_iter().enumerate() {
        println!("*{} - {}", idx + 1, fat);
    }
    println!();
    Ok(())
}
