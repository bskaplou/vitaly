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
            //println!("loading meta from file {:?}", &meta_file);
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
    encoders: &Vec<protocol::Encoder>,
    buttons: &Vec<keymap::Button>,
    layer_number: u8,
    vial_version: u32,
    custom_keycodes: &Option<&Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut button_labels = HashMap::new();

    let custom = if let Some(custom) = custom_keycodes {
        match custom {
            Value::Array(custom) => {
                let mut result: Vec<&str> = Vec::new();
                for code in custom {
                    let name = code
                        .as_object()
                        .ok_or("customKeycode elements should be objects")?
                        .get("shortName")
                        .ok_or("shortName should be defined")?
                        .as_str()
                        .ok_or("shortName should be a string")?;
                    result.push(name);
                }
                result
            }
            // badly formatted json is ignored silently
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // keys wire positons might appear more then once in layout we process them strictly once here
    let mut processed = HashMap::new();
    let mut fat_labels = Vec::new();
    for button in buttons {
        if !button.encoder {
            let wkey = (button.wire_x, button.wire_y);
            if let std::collections::hash_map::Entry::Vacant(e) = processed.entry(wkey) {
                e.insert(true);
                let mut label =
                    keys.get_short(layer_number, button.wire_x, button.wire_y, vial_version)?;
                if let Some(custom_index) = keycodes::is_custom(
                    keys.get(layer_number, button.wire_x, button.wire_y),
                    vial_version,
                ) && custom.len() > custom_index.into()
                {
                    label = custom[custom_index as usize].to_string();
                }
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
                            //println!(
                            //    "{:?} , {:?} at {} {}",
                            //    fat_labels, label, button.wire_x, button.wire_y
                            //);
                            button_labels
                                .insert((button.wire_x, button.wire_y), format!("*{}", pos + 1));
                        }
                    }
                } else {
                    button_labels.insert((button.wire_x, button.wire_y), label.to_string());
                }
            }
        }
    }
    println!("Layer: {}", layer_number);
    keymap::render_and_dump(buttons, Some(button_labels));
    for (idx, fat) in fat_labels.into_iter().enumerate() {
        println!("*{} - {}", idx + 1, fat);
    }
    for e in encoders {
        println!(
            "{0}↺ - {1}\n{0}↻ - {2}",
            e.index,
            keycodes::qid_to_name(e.ccw, vial_version),
            keycodes::qid_to_name(e.cw, vial_version),
        );
    }
    println!();
    Ok(())
}
