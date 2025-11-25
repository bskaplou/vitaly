use crate::common;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    file: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;

    let uid: u64 = protocol::load_uid(&dev)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or("matrix/cols not found in meta")? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or("matrix/rows not found in meta")? as u8;

    let keys = protocol::load_layers_keys(&dev, capabilities.layer_count, rows, cols)?;
    let combos = match capabilities.combo_count {
        0 => Vec::new(),
        _ => protocol::load_combos(&dev, capabilities.combo_count)?,
    };
    let tap_dances = match capabilities.tap_dance_count {
        0 => Vec::new(),
        _ => protocol::load_tap_dances(&dev, capabilities.tap_dance_count)?,
    };
    let macros = protocol::load_macros(
        &dev,
        capabilities.macro_count,
        capabilities.macro_buffer_size,
    )?;

    let key_overrides = match capabilities.key_override_count {
        0 => Vec::new(),
        _ => protocol::load_key_overrides(&dev, capabilities.key_override_count)?,
    };

    let alt_repeats = match capabilities.alt_repeat_key_count {
        0 => Vec::new(),
        _ => protocol::load_alt_repeats(&dev, capabilities.alt_repeat_key_count)?,
    };

    let qmk_settings = if capabilities.vial_version >= protocol::VIAL_PROTOCOL_QMK_SETTINGS {
        protocol::load_qmk_settings(&dev)?
    } else {
        HashMap::new()
    };

    let mut result = json!({
        "version": 1,
        "via_protocol": capabilities.via_version,
        "uid": uid,
        "layout": keys.to_json()?,
    });

    if capabilities.vial_version > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "vial_protocol".to_string(),
            capabilities.vial_version.into(),
        );
    }

    if !alt_repeats.is_empty() {
        result.as_object_mut().ok_or("broken root")?.insert(
            "alt_repeat_key".to_string(),
            Value::Array(protocol::alt_repeats_to_json(&alt_repeats)?),
        );
    }

    if !key_overrides.is_empty() {
        result.as_object_mut().ok_or("broken root")?.insert(
            "key_override".to_string(),
            Value::Array(protocol::key_overrides_to_json(&key_overrides)?),
        );
    }

    if !combos.is_empty() {
        result.as_object_mut().ok_or("broken root")?.insert(
            "combo".to_string(),
            Value::Array(protocol::combos_to_json(&combos)?),
        );
    }

    if !tap_dances.is_empty() {
        result.as_object_mut().ok_or("broken root")?.insert(
            "tap_dance".to_string(),
            Value::Array(protocol::tap_dances_to_json(&tap_dances)?),
        );
    }

    if !macros.is_empty() {
        result.as_object_mut().ok_or("broken root")?.insert(
            "macro".to_string(),
            Value::Array(protocol::macros_to_json(&macros)?),
        );
    }

    if !qmk_settings.is_empty() {
        result.as_object_mut().ok_or("broken root")?.insert(
            "settings".to_string(),
            protocol::qmk_settings_to_json(&qmk_settings)?,
        );
    }

    fs::write(file, result.to_string())?;
    println!("\nConfigutaion saved to file {}", file);
    Ok(())
}
