use crate::keycodes;
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
    encoders: &HashMap<u8, (u16, u16)>,
    buttons: &Vec<keymap::Button>,
    layer_number: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut button_labels = HashMap::new();
    // keys wire positons might appear more then once in layout we process them strictly once here
    let mut processed = HashMap::new();
    let mut fat_labels = Vec::new();
    for button in buttons {
        if !button.encoder {
            let wkey = (button.wire_x, button.wire_y);
            if let std::collections::hash_map::Entry::Vacant(e) = processed.entry(wkey) {
                e.insert(true);
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
                            println!(
                                "{:?} , {:?} at {} {}",
                                fat_labels, label, button.wire_x, button.wire_y
                            );
                            button_labels
                                .insert((button.wire_x, button.wire_y), format!("*{}", pos));
                        }
                    }
                } else {
                    button_labels.insert((button.wire_x, button.wire_y), label);
                }
            }
        }
    }
    println!("Layer: {}", layer_number);
    keymap::render_and_dump(buttons, Some(button_labels));
    for (idx, fat) in fat_labels.into_iter().enumerate() {
        println!("*{} - {}", idx + 1, fat);
    }
    for (idx, (ccw, cw)) in encoders {
        println!(
            "{0}↺ - {1}\n{0}↻ - {2}",
            idx,
            keycodes::qid_to_name(*ccw),
            keycodes::qid_to_name(*cw)
        );
    }
    println!();
    Ok(())
}
