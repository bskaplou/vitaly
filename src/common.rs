use crate::keycodes;
use crate::keymap;
use crate::protocol;
use anyhow::{Result, anyhow};
use hidapi::HidDevice;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
#[error("{0}")]
pub struct CommandError(pub String);

fn load_via_meta_ref(vendor_id: u16, product_id: u16) -> Result<Option<String>> {
    let link_head = "https://raw.githubusercontent.com/the-via/keyboards/refs/heads/master/src/";
    let via_meta_json = include_str!("via_meta_refs.json");
    let vial_meta: Value = serde_json::from_str(via_meta_json)?;
    let key = format!("{:#06x}_{:#06x}", vendor_id, product_id);
    if let Some(path) = vial_meta
        .as_object()
        .ok_or(anyhow!("bad meta-dict file"))?
        .get(&key)
    {
        let result = format!(
            "{link_head}{}",
            path.as_str().ok_or(anyhow!("bad meta-dict format"))?
        );
        Ok(Some(result.to_string()))
    } else {
        Ok(None)
    }
}

fn load_via_meta(dev: &HidDevice) -> Result<String> {
    let di = dev.get_device_info()?;
    let url = load_via_meta_ref(di.vendor_id(), di.product_id())?;
    if let Some(url) = url {
        println!("Using default keyboard specification file {}", url);
        let meta_str = ureq::get(url).call()?.body_mut().read_to_string()?;
        Ok(meta_str.to_string())
    } else {
        Err(anyhow!(
            "failed to find external meta file, please find metadata file and pass it with -m argument"
        ))
    }
}

pub fn load_meta(
    dev: &HidDevice,
    capabilities: &protocol::Capabilities,
    meta_file: &Option<String>,
) -> Result<Value> {
    match meta_file {
        Some(meta_file) => {
            //println!("loading meta from file {:?}", &meta_file);
            let meta_str = fs::read_to_string(meta_file)?;
            Ok(serde_json::from_str(&meta_str)?)
        }
        None => {
            if capabilities.vial_version == 0 {
                let meta_str = load_via_meta(dev)?;
                let meta: Value = serde_json::from_str(&meta_str)?;
                Ok(meta)
            } else {
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
}

pub struct LayerRenderData {
    pub buffer: keymap::Buffer,
    pub fat_labels: Vec<String>,
}

type LayerLabels = (HashMap<(u8, u8), String>, Vec<String>);

fn process_layer_labels(
    keys: &protocol::Keymap,
    buttons: &Vec<keymap::Button>,
    layer_number: u8,
    vial_version: u32,
    custom_keycodes: &Option<&Value>,
) -> Result<LayerLabels> {
    let mut button_labels = HashMap::new();

    let custom = if let Some(custom) = custom_keycodes {
        match custom {
            Value::Array(custom) => {
                let mut result: Vec<String> = Vec::new();
                for (idx, code) in custom.iter().enumerate() {
                    let custom_keycode = code
                        .as_object()
                        .ok_or(anyhow!("customKeycode elements should be objects"))?;
                    let mut name = custom_keycode.get("shortName");
                    if name.is_none() {
                        name = custom_keycode.get("name");
                    }
                    let name = name
                        .ok_or(anyhow!("shortName or name should be defined"))?
                        .as_str()
                        .ok_or(anyhow!("shortName/name should be a string"))?
                        .replace('\n', " ");
                    result.push(format!("QK_KB_{} - {}", idx, name));
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
    Ok((button_labels, fat_labels))
}

pub fn prepare_layer_render(
    keys: &protocol::Keymap,
    buttons: &Vec<keymap::Button>,
    layer_number: u8,
    vial_version: u32,
    custom_keycodes: &Option<&Value>,
) -> Result<LayerRenderData> {
    let (button_labels, fat_labels) =
        process_layer_labels(keys, buttons, layer_number, vial_version, custom_keycodes)?;

    let buff = keymap::render_to_buffer(buttons, Some(button_labels));

    Ok(LayerRenderData {
        buffer: buff,
        fat_labels,
    })
}

pub fn render_layer(
    keys: &protocol::Keymap,
    encoders: &Vec<protocol::Encoder>,
    buttons: &Vec<keymap::Button>,
    layer_number: u8,
    vial_version: u32,
    custom_keycodes: &Option<&Value>,
) -> Result<()> {
    let (button_labels, fat_labels) =
        process_layer_labels(keys, buttons, layer_number, vial_version, custom_keycodes)?;

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
