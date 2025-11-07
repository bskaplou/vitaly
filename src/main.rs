extern crate hidapi;

use argh::FromArgs;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

pub mod keymap;
pub mod protocol;

/// VIA/Vial HID USB cli tool
#[derive(FromArgs)]
struct VialClient {
    /// device product id
    #[argh(option, short = 'i')]
    id: Option<u16>,

    /// command to run
    #[argh(subcommand)]
    command: CommandEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum CommandEnum {
    Devices(CommandDevices),
    Settings(CommandSettings),
    Layers(CommandLayers),
    Keys(CommandKeys),
    Combos(CommandCombos),
    Macros(CommandMacros),
    TapDances(CommandTapDances),
    KeyOverrides(CommandKeyOverrides),
    AltRepeats(CommandAltRepeats),
}

#[derive(FromArgs, PartialEq, Debug)]
/// List connected devices
#[argh(subcommand, name = "devices")]
struct CommandDevices {
    /// scan for capabilities
    #[argh(switch, short = 'c')]
    capabilities: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Combos operations
#[argh(subcommand, name = "combos")]
struct CommandCombos {
    /// combo number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format KEY_1 + KEY_2 + KEY_3 + KEY_4 = KEY_5
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Macros operations
#[argh(subcommand, name = "macros")]
struct CommandMacros {
    /// macro number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format UNKNOWN
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// TapDance operations
#[argh(subcommand, name = "tapdances")]
struct CommandTapDances {
    /// tap dance number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format TAP_KEY + HOLD_KEY + DOUBLE_TAP_KEY + TAPHOLD_KEY ~ TAPPING_TERM_MS
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// KeyOverride operations
#[argh(subcommand, name = "keyoverrides")]
struct CommandKeyOverrides {
    /// tap dance number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format trigger=KC_1; replacement=KC_2; layers=1|2|3; trigger_mods=LS|RS; negative_mod_mask=LC|RC; suppressed_mods =LGUI|RGUI; options=ko_enabled|ko_option_activation_trigger_down
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// AltRepeat operations
#[argh(subcommand, name = "altrepeats")]
struct CommandAltRepeats {
    /// alt repeat number
    #[argh(option, short = 'n')]
    number: Option<u8>,

    /// value expression in format keycode=KC_1; alt_keycode=KC_2; allowed_mods=LS; options=arep_enabled   
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Layers operations
#[argh(subcommand, name = "layers")]
struct CommandLayers {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// show positions instead of assignments
    #[argh(switch, short = 'p')]
    positions: bool,

    /// layer number
    #[argh(option, short = 'n')]
    number: Option<u8>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Key operations
#[argh(subcommand, name = "keys")]
struct CommandKeys {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// key layer
    #[argh(option, short = 'l')]
    layer: u8,

    /// key position
    #[argh(option, short = 'p')]
    position: String,

    /// key value
    #[argh(option, short = 'v')]
    value: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Settings operations
#[argh(subcommand, name = "settings")]
struct CommandSettings {
    /// setting identifier
    #[argh(option, short = 'q')]
    qsid: Option<f64>,

    /// set setting value
    #[argh(option, short = 'v')]
    value: Option<String>,

    /// reset all settings into default values
    #[argh(switch, short = 'r')]
    reset: bool,
}

#[allow(dead_code)]
#[derive(Error, Debug)]
#[error("{0}")]
pub struct CommandError(String);

fn run_devices(
    api: &HidApi,
    device: &DeviceInfo,
    capabilities: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if capabilities {
        let device_path = device.path();
        let dev = api.open_path(device_path)?;
        let capabilities = protocol::scan_capabilities(&dev)?;
        println!("Capabilities:\n\tvia_version: {}", capabilities.via_version);
        println!("\tvial_version: {}", capabilities.vial_version);
        println!(
            "\tcompanion_hid_version: {}",
            capabilities.companion_hid_version
        );
        println!("\tlayer_count: {}", capabilities.layer_count);
        println!("\tmacro_count: {}", capabilities.macro_count);
        println!("\tmacro_buffer_size: {}", capabilities.macro_buffer_size);
        println!("\ttap_dance_count: {}", capabilities.tap_dance_count);
        println!("\tcombo_count: {}", capabilities.combo_count);
        println!("\tkey_override_count: {}", capabilities.key_override_count);
        println!(
            "\talt_repeat_key_count: {}",
            capabilities.alt_repeat_key_count
        );
        println!("\tcaps_word: {}", capabilities.caps_word);
        println!("\tlayer_lock: {}", capabilities.layer_lock);
    }
    println!("");
    Ok(())
}

fn run_combos(
    api: &HidApi,
    device: &DeviceInfo,
    number: Option<u8>,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;

    if capabilities.combo_count == 0 {
        return Err(CommandError(
            format!("device {:?} has doesn't support combos", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.combo_count {
                return Err(CommandError(
                    format!("Only {} combo avialable", capabilities.combo_count).to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    match value {
        None => {
            let combos = protocol::load_combos(&dev, capabilities.combo_count)?;
            if matches!(number, None) {
                let combo_count = combos.len() - 1;
                let mut last_non_empty = capabilities.combo_count;
                for idxm in 0..=combo_count {
                    let idx = combo_count - idxm;
                    last_non_empty = idx as u8;
                    if !combos[idx as usize].empty() {
                        break;
                    }
                }
                println!("Combos list:");
                for idx in 0..=last_non_empty {
                    println!("{}", combos[idx as usize]);
                }
                if last_non_empty < capabilities.combo_count {
                    println!(
                        "Combo slots {} - {} are EMPTY",
                        last_non_empty + 1,
                        capabilities.combo_count - 1
                    );
                }
            } else {
                println!("{}", combos[n as usize]);
            }
        }
        Some(value) => {
            let (keys_all, output) = value.split_once("=").unwrap();
            let keys: Vec<_> = keys_all.split("+").collect();
            match protocol::Combo::from_strings(n, keys, output) {
                Ok(combo) => {
                    println!("Saving combo {}", combo);
                    match protocol::set_combo(&dev, &combo) {
                        Ok(_) => {
                            // nothing here
                        }
                        Err(e) => {
                            println!("Failed to save combo {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse combo {:?}", e);
                }
            }
        }
    }
    Ok(())
}

fn run_macros(
    api: &HidApi,
    device: &DeviceInfo,
    number: Option<u8>,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;

    if capabilities.macro_count == 0 {
        return Err(CommandError(
            format!("device {:?} has doesn't support macros", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.macro_count {
                return Err(CommandError(
                    format!("Only {} macros avialable", capabilities.macro_count).to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    match value {
        None => {
            let macros = protocol::load_macros(
                &dev,
                capabilities.macro_count,
                capabilities.macro_buffer_size,
            )?;
            for m in macros {
                println!("{:?}", m)
            }
        }
        Some(value) => todo!(),
    }
    Ok(())
}

fn parse_num(num: &String) -> Result<u16, CommandError> {
    let parsed: Result<u16, _> = num.parse();
    match parsed {
        Ok(p) => Ok(p),
        Err(_) => Err(CommandError(
            format!("can't argument {} should be num", num).to_string(),
        )),
    }
}

fn run_tapdances(
    api: &HidApi,
    device: &DeviceInfo,
    number: Option<u8>,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    if capabilities.tap_dance_count == 0 {
        return Err(CommandError(
            format!("device {:?} has doesn't support tap dance", device).to_string(),
        )
        .into());
    }
    let n: u8;
    match number {
        Some(num) => {
            n = num;
            if n >= capabilities.tap_dance_count {
                return Err(CommandError(
                    format!("Only {} tap dances avialable", capabilities.tap_dance_count)
                        .to_string(),
                )
                .into());
            }
        }
        None => n = 0,
    }
    match value {
        None => {
            let tapdances = protocol::load_tap_dances(&dev, capabilities.tap_dance_count)?;
            if matches!(number, None) {
                let tapdance_count = tapdances.len() - 1;
                let mut last_non_empty = capabilities.tap_dance_count;
                for idxm in 0..=tapdance_count {
                    let idx = tapdance_count - idxm;
                    last_non_empty = idx as u8;
                    if !tapdances[idx as usize].empty() {
                        break;
                    }
                }
                println!("TapDance list:");
                for idx in 0..=last_non_empty {
                    println!("{}", tapdances[idx as usize]);
                }
                if last_non_empty < capabilities.tap_dance_count {
                    println!(
                        "TapDance slots {} - {} are EMPTY",
                        last_non_empty + 1,
                        capabilities.tap_dance_count - 1
                    );
                }
            } else {
                println!("{}", tapdances[n as usize]);
            }
        }
        Some(value) => {
            let (keys_all, output) = value.split_once("~").unwrap();
            let out: u16 = parse_num(&output.replace(" ", ""))?;
            let keys: Vec<_> = keys_all.split("+").collect();
            match protocol::TapDance::from_strings(n, keys, out) {
                Ok(tapdance) => {
                    println!("Saving tap dance {}", tapdance);
                    match protocol::set_tap_dance(&dev, &tapdance) {
                        Ok(_) => {
                            // nothing here
                        }
                        Err(e) => {
                            println!("Failed to save tap dance {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse tap dance {:?}", e);
                }
            }
        }
    }
    Ok(())
}

fn run_altrepeats(
    api: &HidApi,
    device: &DeviceInfo,
    number: Option<u8>,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    if capabilities.alt_repeat_key_count == 0 {
        return Err(CommandError(
            format!("device {:?} has doesn't support alt repeat keys", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.alt_repeat_key_count {
                return Err(CommandError(
                    format!(
                        "Only {} alt repleat keys avialable",
                        capabilities.alt_repeat_key_count
                    )
                    .to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    match value {
        Some(value) => {
            match protocol::AltRepeat::from_strings(n, value.replace(" ", "").split(";").collect())
            {
                Ok(ar) => {
                    println!("Saving altrepeat {}", ar);
                    match protocol::set_alt_repeat(&dev, &ar) {
                        Ok(_) => {
                            // nothing here
                        }
                        Err(e) => {
                            println!("Failed to save altrepeat {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse altrepeat {:?}", e);
                }
            }
        }
        None => {
            let altrepeats = protocol::load_alt_repeats(&dev, capabilities.alt_repeat_key_count)?;
            if matches!(number, None) {
                println!("AltRepeat list:");
                for idx in 0..capabilities.alt_repeat_key_count {
                    println!("{}", altrepeats[idx as usize]);
                }
            } else {
                println!("{}", altrepeats[n as usize]);
            }
        }
    }
    Ok(())
}

fn run_keyoverrides(
    api: &HidApi,
    device: &DeviceInfo,
    number: Option<u8>,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    if capabilities.key_override_count == 0 {
        return Err(CommandError(
            format!("device {:?} has doesn't support key override", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.key_override_count {
                return Err(CommandError(
                    format!(
                        "Only {} key overrides avialable",
                        capabilities.key_override_count
                    )
                    .to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    match value {
        Some(value) => match protocol::KeyOverride::from_strings(
            n,
            value.replace(" ", "").split(";").collect(),
        ) {
            Ok(ko) => {
                protocol::set_key_override(&dev, &ko)?;
                println!("Key override {}\nSaved", ko);
            }
            Err(e) => {
                println!("failed to parse key override {:?}", e);
            }
        },
        None => {
            let keyoverrides = protocol::load_key_overrides(&dev, capabilities.key_override_count)?;
            if matches!(number, None) {
                println!("KeyOverride list:");
                for idx in 0..capabilities.key_override_count {
                    println!("{}", keyoverrides[idx as usize]);
                }
            } else {
                println!("{}", keyoverrides[n as usize]);
            }
        }
    }

    Ok(())
}

fn load_meta(
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
                    format!("device doesn't support vial protocol").to_string(),
                )
                .into());
            }
            let meta_data = match protocol::load_vial_meta(&dev) {
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

fn run_layers(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    positions: bool,
    number: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = load_meta(&dev, &capabilities, &meta_file)?;
    let buttons = match keymap::keymap_to_buttons(&meta["layouts"]["keymap"]) {
        Ok(btns) => btns,
        Err(e) => {
            return Err(
                CommandError(format!("failed to process keymaps {:?}", e).to_string()).into(),
            );
        }
    };
    if positions == true {
        keymap::render_and_dump(buttons, None);
    } else {
        let layer_number: u8;
        match number {
            Some(n) => layer_number = n,
            None => layer_number = 0,
        }
        let cols = meta["matrix"]["cols"].as_u64().unwrap() as u8;
        let rows = meta["matrix"]["rows"].as_u64().unwrap() as u8;
        let keys = protocol::load_layers_keys(&dev, capabilities.layer_count, rows, cols)?;
        let mut button_labels = HashMap::new();
        let mut fat_labels = Vec::new();
        for button in &buttons {
            let label = keys.get_short(layer_number, button.wire_x, button.wire_y)?;
            let mut slim_label = true;
            for (idx, part) in label.split(',').enumerate() {
                if part.chars().count() > 3 || idx > 1 {
                    slim_label &= false;
                }
            }
            if !slim_label {
                fat_labels.push(label);
                button_labels.insert(
                    (button.wire_x, button.wire_y),
                    format!("*{}", fat_labels.len()),
                );
            } else {
                button_labels.insert((button.wire_x, button.wire_y), label);
            }
        }
        println!("Layer: {}", layer_number);
        keymap::render_and_dump(buttons, Some(button_labels));
        for (idx, fat) in fat_labels.into_iter().enumerate() {
            println!("*{} - {}", idx + 1, fat);
        }
    }
    Ok(())
}

fn run_keys(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    layer: u8,
    position: &String,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = load_meta(&dev, &capabilities, &meta_file)?;
    let cols = meta["matrix"]["cols"].as_u64().unwrap() as u8;
    let rows = meta["matrix"]["rows"].as_u64().unwrap() as u8;
    if position.split(',').count() != 2 {
        return Err(CommandError(
            format!(
                "position {:?} is incorrect, should be in format row,col for example 0,1",
                position
            )
            .to_string(),
        )
        .into());
    }
    let mut p_parts = position.split(',');
    let row: u8 = p_parts.next().unwrap().parse()?;
    let col: u8 = p_parts.next().unwrap().parse()?;
    match value {
        Some(value) => match protocol::keycodes::name_to_qid(value) {
            Ok(keycode) => {
                protocol::set_keycode(&dev, layer, row, col, keycode)?;
                println!(
                    "Key on layer={:?}, row={:?}, col={:?} set to => {}, keycode = {:#x}",
                    layer, row, col, value, keycode,
                );
            }
            Err(e) => {
                return Err(
                    CommandError(format!("failed to build keycode {:?}", e).to_string()).into(),
                );
            }
        },
        None => {
            let keys = protocol::load_layers_keys(&dev, capabilities.layer_count, rows, cols)?;
            let label = keys.get_long(layer, row, col)?;
            println!(
                "Key on layer={:?}, row={:?}, col={:?} => {}",
                layer, row, col, label
            );
        }
    }

    Ok(())
}

fn run_settings(
    api: &HidApi,
    device: &DeviceInfo,
    qsid: &Option<f64>,
    value: &Option<String>,
    reset: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    if capabilities.vial_version < protocol::VIAL_PROTOCOL_QMK_SETTINGS {
        return Err(CommandError("Qmk settings are not supported".to_string()).into());
    }
    if reset {
        if !matches!(qsid, None) || !matches!(value, None) {
            return Err(CommandError(
                "Values can be reset only all at once, no qsid nor value should be passed"
                    .to_string(),
            )
            .into());
        }
        protocol::reset_qmk_values(&dev)?;
        return Ok(());
    }
    let qsids = protocol::load_qmk_qsids(&dev)?;
    let settings = protocol::load_qmk_definitions()?;
    match qsid {
        Some(qsid_full) => {
            let qsid_full_str = qsid_full.to_string();
            let tsid: u16;
            let tbit: u8;
            if qsid_full_str.contains('.') {
                let (l, r) = qsid_full_str.split_once('.').unwrap();
                tsid = l.parse()?;
                tbit = r.parse()?;
            } else {
                tsid = qsid_full_str.parse()?;
                tbit = 0;
            }
            for group in settings["tabs"].as_array().unwrap() {
                //let group_name = group["name"].as_str().unwrap();
                for field in group["fields"].as_array().unwrap() {
                    let qsid = field["qsid"].as_u64().unwrap() as u16;
                    let title = field["title"].as_str().unwrap();
                    let width: u8;
                    match &field["width"] {
                        Value::Number(n) => width = n.as_u64().unwrap() as u8,
                        _ => width = 1,
                    }
                    let bool_field = field["type"].as_str().unwrap() == "boolean";
                    let with_bits = !matches!(field["bit"], Value::Null);
                    if qsid == tsid
                        && (with_bits == false || (field["bit"].as_u64().unwrap() as u8) == tbit)
                    {
                        match value {
                            None => {
                                let value = protocol::get_qmk_value(&dev, qsid, width)?;
                                if bool_field {
                                    if with_bits {
                                        println!(
                                            "{}.{}) {} = {}",
                                            qsid,
                                            tbit,
                                            title,
                                            value.get_bool(tbit)
                                        );
                                    } else {
                                        println!("{}) {} = {}", qsid, title, value.get() != 0);
                                    }
                                } else {
                                    println!("{}) {} = {}", qsid, title, value.get());
                                }
                            }
                            Some(v) => {
                                if with_bits {
                                    let mut current_value =
                                        protocol::get_qmk_value(&dev, qsid, width)?.get();
                                    let bw: bool = v.parse()?;
                                    if bw {
                                        current_value |= 1 << tbit;
                                    } else {
                                        current_value ^= 1 << tbit;
                                    }
                                    protocol::set_qmk_value(&dev, qsid, current_value)?;
                                } else if bool_field {
                                    let val: bool = v.parse()?;
                                    let int_val = match val {
                                        true => 1,
                                        false => 0,
                                    };
                                    protocol::set_qmk_value(&dev, qsid, int_val)?;
                                } else {
                                    protocol::set_qmk_value(&dev, qsid, v.parse()?)?;
                                }
                                println!("Option {:?} = {} now", title, v);
                            }
                        }
                    }
                }
            }
        }
        None => {
            let mut values_cache = HashMap::new();

            for group in settings["tabs"].as_array().unwrap() {
                let group_name = group["name"].as_str().unwrap();
                println!("\n{}:", group_name);
                for field in group["fields"].as_array().unwrap() {
                    let width: u8;
                    match &field["width"] {
                        Value::Number(n) => width = n.as_u64().unwrap() as u8,
                        _ => width = 1,
                    }
                    let title = field["title"].as_str().unwrap();
                    let qsid = field["qsid"].as_u64().unwrap() as u16;
                    if qsids.contains(&qsid) {
                        let value;
                        if values_cache.contains_key(&qsid) {
                            value = *values_cache.get(&qsid).unwrap();
                        } else {
                            value = protocol::get_qmk_value(&dev, qsid, width)?;
                            values_cache.insert(qsid, value);
                        }
                        match field["type"].as_str().unwrap() {
                            "boolean" => match field["bit"].as_number() {
                                Some(n) => {
                                    let pos = n.as_u64().unwrap() as u8;
                                    println!(
                                        "\t{}.{}) {} = {}",
                                        qsid,
                                        pos,
                                        title,
                                        value.get_bool(n.as_u64().unwrap() as u8)
                                    );
                                }
                                None => {
                                    println!("\t{}) {} = {}", qsid, title, value.get() != 0);
                                }
                            },
                            _ => {
                                println!("\t{}) {} = {}", qsid, title, value.get());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn command_for_devices(id: Option<u16>, command: &CommandEnum) {
    match HidApi::new() {
        Ok(api) => {
            for device in api.device_list() {
                if device.usage_page() == protocol::USAGE_PAGE
                    && device.usage() == protocol::USAGE_ID
                    && (!matches!(id, Some(_)) || id.unwrap() == device.product_id())
                {
                    println!(
                        "Product name: {:?} id: {:?},\nManufacturer name: {:?}, id: {:?},\nRelease: {:?}, Serial: {:?}, Path: {:?}",
                        device.product_string().unwrap(),
                        device.product_id(),
                        device.manufacturer_string().unwrap(),
                        device.vendor_id(),
                        device.release_number(),
                        device.serial_number().unwrap(),
                        device.path(),
                    );
                    let result = match command {
                        CommandEnum::Devices(ops) => run_devices(&api, device, ops.capabilities),
                        CommandEnum::Combos(ops) => {
                            run_combos(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::Macros(ops) => {
                            run_macros(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::TapDances(ops) => {
                            run_tapdances(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::KeyOverrides(ops) => {
                            run_keyoverrides(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::AltRepeats(ops) => {
                            run_altrepeats(&api, device, ops.number, &ops.value)
                        }
                        CommandEnum::Layers(ops) => {
                            run_layers(&api, device, &ops.meta, ops.positions, ops.number)
                        }
                        CommandEnum::Keys(ops) => run_keys(
                            &api,
                            device,
                            &ops.meta,
                            ops.layer,
                            &ops.position,
                            &ops.value,
                        ),
                        CommandEnum::Settings(ops) => {
                            run_settings(&api, device, &ops.qsid, &ops.value, ops.reset)
                        }
                    };
                    match result {
                        Ok(_) => {
                            // nothing here
                        }
                        Err(e) => {
                            println!("Error: {}", e)
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

fn main() {
    let options: VialClient = argh::from_env();
    command_for_devices(options.id, &options.command);
}
