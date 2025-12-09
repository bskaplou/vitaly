use crate::common;
use crate::keymap;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};
use serde_json::Value;
use std::fs;

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    file: &String,
    preview: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or("matrix/cols not found in meta")? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or("matrix/rows not found in meta")? as u8;

    let layout_str = fs::read_to_string(file)?;
    let root_json: Value = serde_json::from_str(&layout_str)?;
    let root = root_json
        .as_object()
        .ok_or("config file root is not an object")?;

    let layout_options = &meta["layouts"]["labels"];
    let layout_state = match &root_json["layout_options"] {
        Value::Null => 0,
        Value::Number(num) => {
            let n = num.as_i64().ok_or("layout_options should be a number")?;
            if n == -1 { 0 } else { n as u32 }
        }
        _ => {
            return Err(
                common::CommandError("layout_option should be a number".to_string()).into(),
            );
        }
    };
    let options = protocol::LayoutOptions::from_json(layout_state, layout_options)?;

    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"], &options)?;

    let layers = root
        .get("layout")
        .ok_or("config file has no layout defined")?
        .as_array()
        .ok_or("layout should be an array")?;

    let keys = protocol::Keymap::from_json(
        rows,
        cols,
        capabilities.layer_count,
        layers,
        capabilities.vial_version,
    )?;
    let encoder_layout = match &root.get("encoder_layout") {
        Some(value) => protocol::load_encoders_from_json(value, capabilities.vial_version)?,
        None => Vec::new(),
    };
    let combos = match capabilities.combo_count {
        0 => Vec::new(),
        _ => protocol::load_combos_from_json(
            root.get("combo").ok_or("combo is not defined")?,
            capabilities.vial_version,
        )?,
    };
    let tap_dances = match capabilities.tap_dance_count {
        0 => Vec::new(),
        _ => protocol::load_tap_dances_from_json(
            root.get("tap_dance").ok_or("tad_dance is not defined")?,
            capabilities.vial_version,
        )?,
    };
    let macros = protocol::load_macros_from_json(
        root.get("macro").ok_or("macro is not defined")?,
        capabilities.vial_version,
    )?;
    let key_overrides = match capabilities.key_override_count {
        0 => Vec::new(),
        _ => protocol::load_key_overrides_from_json(
            root.get("key_override")
                .ok_or("key_override are not defined")?,
            capabilities.vial_version,
        )?,
    };

    let alt_repeats = match capabilities.alt_repeat_key_count {
        0 => Vec::new(),
        _ => protocol::load_alt_repeats_from_json(
            root.get("alt_repeat_key")
                .ok_or("alt_repeat_key are not defined")?,
            capabilities.vial_version,
        )?,
    };

    if !preview {
        println!();
        if !macros.is_empty() && capabilities.vial_version > 0 {
            let status = protocol::get_locked_status(&dev)?;
            if status.locked {
                return Err(common::CommandError("Keyboard is locked, macroses can't be updated, keyboard might be unlocked with subcommand 'lock -u'".to_string()).into());
            }
        }

        protocol::set_layout_options(&dev, layout_state)?;

        protocol::set_macros(&dev, &capabilities, &macros)?;
        println!("Macros restored");

        for ko in key_overrides {
            protocol::set_key_override(&dev, &ko)?;
        }
        println!("Key overrides restored");

        for ar in alt_repeats {
            protocol::set_alt_repeat(&dev, &ar)?;
        }
        println!("Alt repeat keys restored");

        for combo in combos {
            protocol::set_combo(&dev, &combo)?;
        }
        println!("Combos restored");

        for td in tap_dances {
            protocol::set_tap_dance(&dev, &td)?;
        }
        println!("TapDances restored");

        for layer_number in 0..capabilities.layer_count {
            let layer_encoders = if encoder_layout.len() > layer_number.into() {
                &encoder_layout[layer_number as usize]
            } else {
                &Vec::new()
            };
            for encoder in layer_encoders {
                protocol::set_encoder(&dev, layer_number, encoder.index, 0, encoder.ccw)?;
                protocol::set_encoder(&dev, layer_number, encoder.index, 1, encoder.cw)?;
            }
        }
        println!("Encoders restored");

        protocol::set_keymap(&dev, &keys)?;
        println!("Keys restored. All done!!!");

        //
    } else {
        if !options.is_empty() {
            println!("Layout options:\n{}", options);
        }
        for layer_number in 0..capabilities.layer_count {
            let encoders = if encoder_layout.len() > layer_number.into() {
                &encoder_layout[layer_number as usize]
            } else {
                &Vec::new()
            };
            common::render_layer(
                &keys,
                encoders,
                &buttons,
                layer_number,
                capabilities.vial_version,
            )?
        }

        if !combos.is_empty() {
            println!("Combos:");
            for combo in &combos {
                if !combo.is_empty() {
                    combo.dump(capabilities.vial_version)?;
                    println!();
                }
            }
            println!();
        }

        if !macros.is_empty() {
            println!("Macros:");
            for m in &macros {
                if !m.is_empty() {
                    m.dump(capabilities.vial_version)?;
                    println!();
                }
            }
            println!();
        }

        if !tap_dances.is_empty() {
            println!("TapDances:");
            for tap_dance in &tap_dances {
                if !tap_dance.is_empty() {
                    tap_dance.dump(capabilities.vial_version)?;
                    println!();
                }
            }
            println!();
        }

        if !key_overrides.is_empty() {
            println!("KeyOverrides:");
            for key_override in &key_overrides {
                if !key_override.is_empty() {
                    key_override.dump(capabilities.vial_version)?;
                    println!();
                }
            }
            println!();
        }

        if !alt_repeats.is_empty() {
            println!("AltRepeatKeys:");
            for alt_repeat in &alt_repeats {
                if !alt_repeat.is_empty() {
                    alt_repeat.dump(capabilities.vial_version)?;
                    println!();
                }
            }
            println!();
        }

        if capabilities.vial_version >= protocol::VIAL_PROTOCOL_QMK_SETTINGS {
            let qmk_settings = protocol::load_qmk_settings_from_json(
                root.get("settings").ok_or("settings are not defined")?,
            )?;
            let settings_defs = protocol::load_qmk_definitions()?;
            println!("Settings:");
            for group in settings_defs["tabs"]
                .as_array()
                .ok_or("tabs should be an array")?
            {
                let group_name = group["name"].as_str().ok_or("name shoule be a string")?;
                let mut header_shown = false;
                for field in group["fields"]
                    .as_array()
                    .ok_or("fields should be a an array")?
                {
                    let title = field["title"].as_str().ok_or("title should be a string")?;
                    let qsid = field["qsid"].as_u64().ok_or("qsid should be a number")? as u16;
                    if let Some(value) = qmk_settings.get(&qsid) {
                        if !header_shown {
                            println!("{}:", group_name);
                            header_shown = true;
                        }
                        match field["type"].as_str().ok_or("type should be a string")? {
                            "integer" => {
                                println!("\t{}) {} = {}", qsid, title, value.get());
                            }
                            "boolean" => match field["bit"].as_u64() {
                                None => {
                                    println!("\t{}) {} = {}", qsid, title, value.get() != 0);
                                }
                                Some(bit) => {
                                    println!(
                                        "\t{}) {} = {}",
                                        qsid,
                                        title,
                                        value.get_bool(bit as u8)
                                    );
                                }
                            },
                            t => {
                                return Err(common::CommandError(format!(
                                    "Unknown setting type {:?}",
                                    t
                                ))
                                .into());
                            }
                        }
                    }
                }
                if header_shown {
                    println!();
                }
            }
        }
    }
    Ok(())
}
