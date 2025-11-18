extern crate hidapi;

use argh::FromArgs;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::{thread, time};
use thiserror::Error;

pub mod keycodes;
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
    Lock(CommandLock),
    Settings(CommandSettings),
    Layers(CommandLayers),
    Keys(CommandKeys),
    Combos(CommandCombos),
    Macros(CommandMacros),
    TapDances(CommandTapDances),
    KeyOverrides(CommandKeyOverrides),
    AltRepeats(CommandAltRepeats),
    Load(CommandLoad),
    Save(CommandSave),
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
/// List connected devices
#[argh(subcommand, name = "lock")]
struct CommandLock {
    /// scan for capabilities
    #[argh(switch, short = 'u')]
    unlock: bool,
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

    /// value expression in format Text(some text); Tap(KC_1); Down(KC_D); Up(KC_D)
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

#[derive(FromArgs, PartialEq, Debug)]
/// Load layout from file
#[argh(subcommand, name = "load")]
struct CommandLoad {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// path to layout file
    #[argh(option, short = 'f')]
    file: String,

    /// preview content of layout file instead of loading into keyboard
    #[argh(switch, short = 'p')]
    preview: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Load layout from file
#[argh(subcommand, name = "save")]
struct CommandSave {
    /// meta file (to use instead of vial meta)
    #[argh(option, short = 'm')]
    meta: Option<String>,

    /// path to layout file
    #[argh(option, short = 'f')]
    file: String,
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

fn run_lock(
    api: &HidApi,
    device: &DeviceInfo,
    unlock: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = load_meta(&dev, &capabilities, &None)?;
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
            protocol::start_unlock(&dev)?;
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
                let combo_count = combos.len();
                let mut first_empty = capabilities.combo_count;
                for idxm in 1..=combo_count {
                    let idx = combo_count - idxm;
                    if !combos[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("Combos list:");
                for idx in 0..first_empty {
                    println!("{}", combos[idx as usize]);
                }
                if first_empty < capabilities.combo_count {
                    println!(
                        "Combo slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.combo_count - 1
                    );
                }
            } else {
                println!("{}", combos[n as usize]);
            }
        }
        Some(value) => {
            let combo = match value.len() {
                0 => protocol::Combo::empty(n),
                _ => {
                    let (keys_all, output) = value
                        .split_once("=")
                        .ok_or("resulting action should be declared after =")?;
                    let keys: Vec<_> = keys_all.split("+").collect();
                    protocol::Combo::from_strings(n, keys, output)?
                }
            };
            protocol::set_combo(&dev, &combo)?;
            println!("Combo {} saved", combo);
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
    let mut macros = protocol::load_macros(
        &dev,
        capabilities.macro_count,
        capabilities.macro_buffer_size,
    )?;
    match value {
        None => {
            if matches!(number, None) {
                println!("Macros list:");
                for m in macros {
                    println!("{}", m)
                }
            } else {
                if macros.len() > n.into() {
                    println!("{}", macros[n as usize])
                } else {
                    return Err(
                        CommandError(format!("Macro {} is not defined", n).to_string()).into(),
                    );
                }
            }
        }
        Some(value) => {
            let parts = value.split(";").map(|s| s.trim()).collect();
            let m = protocol::Macro::from_strings(n, parts)?;
            if !m.is_empty() {
                if (n as usize) < macros.len() {
                    macros[n as usize] = m;
                } else {
                    macros.push(m);
                }
            } else {
                if (n as usize) < macros.len() {
                    macros.remove(n as usize);
                } else {
                    return Err(CommandError(
                        format!("Can't delete undefined macro {}", n).to_string(),
                    )
                    .into());
                }
            }
            println!("Updated macros list:");
            for m in &macros {
                println!("{}", m)
            }
            if capabilities.vial_version > 0 {
                let status = protocol::get_locked_status(&dev)?;
                if status.locked {
                    return Err(CommandError("Keyboard is locked, macroses can't be updated, keyboard might be unlocked with subcommand 'lock -u'".to_string()).into());
                }
            }
            protocol::set_macros(&dev, &capabilities, &macros)?;
            println!("Macros successfully updated");
        }
    }
    Ok(())
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
                let tapdance_count = tapdances.len();
                let mut first_empty = capabilities.tap_dance_count;
                for idxm in 1..=tapdance_count {
                    let idx = tapdance_count - idxm;
                    if !tapdances[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("TapDance list:");
                for idx in 0..first_empty {
                    println!("{}", tapdances[idx as usize]);
                }
                if first_empty < capabilities.tap_dance_count {
                    println!(
                        "TapDance slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.tap_dance_count - 1
                    );
                }
            } else {
                println!("{}", tapdances[n as usize]);
            }
        }
        Some(value) => {
            let tapdance = match value.len() {
                0 => protocol::TapDance::empty(n),
                _ => {
                    let (keys_all, output) = value
                        .split_once("~")
                        .ok_or("tapping term in ms should be passed after ~")?;
                    let out: u16 = output.replace(" ", "").parse()?;
                    let keys: Vec<_> = keys_all.split("+").collect();
                    protocol::TapDance::from_strings(n, keys, out)?
                }
            };
            protocol::set_tap_dance(&dev, &tapdance)?;
            println!("TapDance {} saved", tapdance);
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
            let alt_repeat = match value.len() {
                0 => protocol::AltRepeat::empty(n),
                _ => {
                    let cleaned = value.replace(" ", "");
                    let parts: Vec<_> = cleaned.split(";").collect();
                    protocol::AltRepeat::from_strings(n, parts)?
                }
            };
            protocol::set_alt_repeat(&dev, &alt_repeat)?;
            println!("AltRepeat {} saved", alt_repeat);
        }
        None => {
            let altrepeats = protocol::load_alt_repeats(&dev, capabilities.alt_repeat_key_count)?;
            if matches!(number, None) {
                let altrepeat_count = altrepeats.len();
                let mut first_empty = capabilities.alt_repeat_key_count;
                for idxm in 1..=altrepeat_count {
                    let idx = altrepeat_count - idxm;
                    if !altrepeats[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("AltRepeat list:");
                for idx in 0..first_empty {
                    println!("{}", altrepeats[idx as usize]);
                }
                if first_empty < capabilities.alt_repeat_key_count {
                    println!(
                        "AltRepeat slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.alt_repeat_key_count - 1
                    );
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
        Some(value) => {
            let ko = match value.len() {
                0 => protocol::KeyOverride::empty(n),
                _ => protocol::KeyOverride::from_strings(
                    n,
                    value.replace(" ", "").split(";").collect(),
                )?,
            };
            protocol::set_key_override(&dev, &ko)?;
            println!("KeyOverride {} saved", ko);
        }
        None => {
            let keyoverrides = protocol::load_key_overrides(&dev, capabilities.key_override_count)?;
            if matches!(number, None) {
                let keyoverride_count = keyoverrides.len();
                let mut first_empty = capabilities.key_override_count;
                for idxm in 1..=keyoverride_count {
                    let idx = keyoverride_count - idxm;
                    if !keyoverrides[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("KeyOverride list:");
                for idx in 0..first_empty {
                    println!("{}", keyoverrides[idx as usize]);
                }
                if first_empty < capabilities.key_override_count {
                    println!(
                        "KeyOverride slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.key_override_count - 1
                    );
                }
            } else {
                println!("{}", keyoverrides[n as usize]);
            }
        }
    }

    Ok(())
}

fn render_layer(
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
    keymap::render_and_dump(&buttons, Some(button_labels));
    for (idx, fat) in fat_labels.into_iter().enumerate() {
        println!("*{} - {}", idx + 1, fat);
    }
    println!("");
    Ok(())
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
    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"])?;
    if positions == true {
        keymap::render_and_dump(&buttons, None);
    } else {
        let layer_number: u8 = match number {
            Some(n) => n,
            None => 0,
        };
        let cols = meta["matrix"]["cols"]
            .as_u64()
            .ok_or("matrix/cols not found in meta")? as u8;
        let rows = meta["matrix"]["rows"]
            .as_u64()
            .ok_or("matrix/rows not found in meta")? as u8;
        let keys = protocol::load_layers_keys(&dev, capabilities.layer_count, rows, cols)?;
        render_layer(&keys, &buttons, layer_number)?
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
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or("matrix/cols not found in meta")? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or("matrix/rows not found in meta")? as u8;

    let row: u8;
    let col: u8;
    if let Some((r, c)) = position.split_once(',') {
        row = r.parse()?;
        col = c.parse()?;
    } else {
        return Err(CommandError("position format is 'row,col'".to_string()).into());
    }
    match value {
        Some(value) => match keycodes::name_to_qid(value) {
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
            if let Some((l, r)) = qsid_full_str.split_once('.') {
                tsid = l.parse()?;
                tbit = r.parse()?;
            } else {
                tsid = qsid_full_str.parse()?;
                tbit = 0;
            }
            for group in settings["tabs"]
                .as_array()
                .ok_or("tabs should be an array")?
            {
                //let group_name = group["name"].as_str().unwrap();
                for field in group["fields"]
                    .as_array()
                    .ok_or("fields should be an array")?
                {
                    let qsid = field["qsid"].as_u64().ok_or("qsid should be a number")? as u16;
                    let title = field["title"].as_str().ok_or("title should be a string")?;
                    let width: u8 = match &field["width"] {
                        Value::Number(n) => n.as_u64().ok_or("width shoulbe a number")? as u8,
                        _ => 1,
                    };
                    let bool_field =
                        field["type"].as_str().ok_or("type should be string")? == "boolean";
                    let with_bits = !matches!(field["bit"], Value::Null);
                    if qsid == tsid
                        && (with_bits == false
                            || (field["bit"].as_u64().ok_or("bit should be number")? as u8) == tbit)
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

            for group in settings["tabs"]
                .as_array()
                .ok_or("tabs should be an array")?
            {
                let group_name = group["name"].as_str().ok_or("name should be a string")?;
                println!("\n{}:", group_name);
                for field in group["fields"]
                    .as_array()
                    .ok_or("fields should be an array")?
                {
                    let width: u8 = match &field["width"] {
                        Value::Number(n) => n.as_u64().ok_or("width should be a number")? as u8,
                        _ => 1,
                    };
                    let title = field["title"].as_str().ok_or("title should be a string")?;
                    let qsid = field["qsid"].as_u64().ok_or("title should be a number")? as u16;
                    if qsids.contains(&qsid) {
                        let value;
                        if values_cache.contains_key(&qsid) {
                            value = *values_cache.get(&qsid).ok_or("cache broken")?;
                        } else {
                            value = protocol::get_qmk_value(&dev, qsid, width)?;
                            values_cache.insert(qsid, value);
                        }
                        match field["type"].as_str().ok_or("type should be a string")? {
                            "boolean" => match field["bit"].as_number() {
                                Some(n) => {
                                    let pos = n.as_u64().ok_or("bit should be a number")? as u8;
                                    println!(
                                        "\t{}.{}) {} = {}",
                                        qsid,
                                        pos,
                                        title,
                                        value.get_bool(pos)
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

fn run_load(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    file: &String,
    preview: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = load_meta(&dev, &capabilities, meta_file)?;
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or("matrix/cols not found in meta")? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or("matrix/rows not found in meta")? as u8;
    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"])?;

    let layout_str = fs::read_to_string(file)?;
    let root_json: Value = serde_json::from_str(&layout_str)?;
    let root = root_json
        .as_object()
        .ok_or("config file root is not object")?;
    let layers = root
        .get("layout")
        .ok_or("config file has no layout defined")?
        .as_array()
        .ok_or("layout should be an array")?;

    let keys = protocol::Keymap::from_json(rows, cols, capabilities.layer_count, layers)?;
    let combos = match capabilities.combo_count {
        0 => Vec::new(),
        _ => protocol::load_combos_from_json(root.get("combo").ok_or("combo is not defined")?)?,
    };
    let tap_dances = match capabilities.tap_dance_count {
        0 => Vec::new(),
        _ => protocol::load_tap_dances_from_json(
            root.get("tap_dance").ok_or("tad_dance is not defined")?,
        )?,
    };
    let macros = protocol::load_macros_from_json(root.get("macro").ok_or("macro is not defined")?)?;

    let key_overrides = match capabilities.key_override_count {
        0 => Vec::new(),
        _ => protocol::load_key_overrides_from_json(
            root.get("key_override")
                .ok_or("key_override are not defined")?,
        )?,
    };

    let alt_repeats = match capabilities.alt_repeat_key_count {
        0 => Vec::new(),
        _ => protocol::load_alt_repeats_from_json(
            root.get("alt_repeat_key")
                .ok_or("alt_repeat_key are not defined")?,
        )?,
    };

    if preview {
        for layer_number in 0..capabilities.layer_count {
            render_layer(&keys, &buttons, layer_number)?
        }

        if combos.len() > 0 {
            println!("Combos:");
            for combo in &combos {
                if !combo.is_empty() {
                    println!("{}", &combo);
                }
            }
            println!("");
        }

        if macros.len() > 0 {
            println!("Macros:");
            for m in &macros {
                if !m.is_empty() {
                    println!("{}", &m);
                }
            }
            println!("");
        }

        if tap_dances.len() > 0 {
            println!("TapDances:");
            for tap_dance in &tap_dances {
                if !tap_dance.is_empty() {
                    println!("{}", &tap_dance);
                }
            }
            println!("");
        }

        if key_overrides.len() > 0 {
            println!("KeyOverrides:");
            for key_override in &key_overrides {
                if !key_override.is_empty() {
                    println!("{}", &key_override);
                }
            }
            println!("");
        }

        if alt_repeats.len() > 0 {
            println!("AltRepeatKeys:");
            for alt_repeat in &alt_repeats {
                if !alt_repeat.is_empty() {
                    println!("{}", &alt_repeat);
                }
            }
            println!("");
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
                                return Err(
                                    CommandError(format!("Unknown setting type {:?}", t)).into()
                                );
                            }
                        }
                    }
                }
                if header_shown {
                    println!("");
                }
            }
        }
    }
    Ok(())
}

fn run_save(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    file: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;

    let uid: u64 = protocol::load_uid(&dev)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = load_meta(&dev, &capabilities, meta_file)?;
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or("matrix/cols not found in meta")? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or("matrix/rows not found in meta")? as u8;
    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"])?;

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
    });

    if capabilities.vial_version > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "vial_protocol".to_string(),
            capabilities.vial_version.into(),
        );
    }

    if alt_repeats.len() > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "alt_repeat_key".to_string(),
            Value::Array(protocol::alt_repeats_to_json(&alt_repeats)?),
        );
    }

    if key_overrides.len() > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "key_override".to_string(),
            Value::Array(protocol::key_overrides_to_json(&key_overrides)?),
        );
    }

    if combos.len() > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "combo".to_string(),
            Value::Array(protocol::combos_to_json(&combos)?),
        );
    }

    if tap_dances.len() > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "tap_dance".to_string(),
            Value::Array(protocol::tap_dances_to_json(&tap_dances)?),
        );
    }

    if macros.len() > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "macro".to_string(),
            Value::Array(protocol::macros_to_json(&macros)?),
        );
    }

    if qmk_settings.len() > 0 {
        result.as_object_mut().ok_or("broken root")?.insert(
            "settings".to_string(),
            protocol::qmk_settings_to_json(&qmk_settings)?,
        );
    }
    // println!("{}", result.to_string());

    fs::write(file, result.to_string())?;
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
                        CommandEnum::Lock(ops) => run_lock(&api, device, ops.unlock),
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
                        CommandEnum::Load(ops) => {
                            run_load(&api, device, &ops.meta, &ops.file, ops.preview)
                        }
                        CommandEnum::Save(ops) => run_save(&api, device, &ops.meta, &ops.file),
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
