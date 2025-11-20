use hidapi::{HidDevice, HidError, HidResult};
use lzma::LzmaError;
use serde_json::Value;
use std::cmp::min;
use std::fmt;
use std::string::FromUtf8Error;
use thiserror::Error;

use crate::keycodes;

pub mod key_override;
pub use crate::protocol::key_override::{
    key_overrides_to_json, load_key_overrides, load_key_overrides_from_json, set_key_override,
    KeyOverride,
};

pub mod alt_repeat;
pub use crate::protocol::alt_repeat::{
    alt_repeats_to_json, load_alt_repeats, load_alt_repeats_from_json, set_alt_repeat, AltRepeat,
};

pub mod tap_dance;
pub use crate::protocol::tap_dance::{
    load_tap_dances, load_tap_dances_from_json, set_tap_dance, tap_dances_to_json, TapDance,
};

pub mod combo;
pub use crate::protocol::combo::{
    combos_to_json, load_combos, load_combos_from_json, set_combo, Combo,
};

pub mod r#macro;
pub use crate::protocol::r#macro::{
    load_macros, load_macros_from_json, macros_to_json, set_macros, Macro,
};

pub mod qmk_settings;
pub use crate::protocol::qmk_settings::{
    get_qmk_value, load_qmk_definitions, load_qmk_qsids, load_qmk_settings,
    load_qmk_settings_from_json, qmk_settings_to_json, reset_qmk_values, set_qmk_value, QmkValue,
};

pub const USAGE_PAGE: u16 = 0xFF60;
pub const USAGE_ID: u16 = 0x61;

const MESSAGE_LENGTH: usize = 32;

const HID_LAYERS_IN: u8 = 0x88;
const GET_VERSION: u8 = 0x00;
const HID_LAYERS_OUT_VERSION: u8 = 0x91;

pub const VIAL_PROTOCOL_DYNAMIC: u32 = 4;
pub const VIAL_PROTOCOL_QMK_SETTINGS: u32 = 4;

const CMD_VIA_GET_PROTOCOL_VERSION: u8 = 0x01;
const CMD_VIA_VIAL_PREFIX: u8 = 0xFE;
const CMD_VIAL_GET_KEYBOARD_ID: u8 = 0x00;
const CMD_VIAL_GET_SIZE: u8 = 0x01;
const CMD_VIAL_GET_DEFINITION: u8 = 0x02;
const CMD_VIA_SET_KEYCODE: u8 = 0x05;
const CMD_VIA_GET_LAYER_COUNT: u8 = 0x11;
const CMD_VIA_KEYMAP_GET_BUFFER: u8 = 0x12;
const VIA_UNHANDLED: u8 = 0xFF;

const CMD_VIAL_DYNAMIC_ENTRY_OP: u8 = 0x0D;
const DYNAMIC_VIAL_GET_NUMBER_OF_ENTRIES: u8 = 0x00;
const DYNAMIC_VIAL_TAP_DANCE_GET: u8 = 0x01;
const DYNAMIC_VIAL_TAP_DANCE_SET: u8 = 0x02;
const DYNAMIC_VIAL_COMBO_GET: u8 = 0x03;
const DYNAMIC_VIAL_COMBO_SET: u8 = 0x04;
const DYNAMIC_VIAL_KEY_OVERRIDE_GET: u8 = 0x05;
const DYNAMIC_VIAL_KEY_OVERRIDE_SET: u8 = 0x06;
const DYNAMIC_VIAL_ALT_REPEAT_KEY_GET: u8 = 0x07;
const DYNAMIC_VIAL_ALT_REPEAT_KEY_SET: u8 = 0x08;
const CMD_VIAL_QMK_SETTINGS_QUERY: u8 = 0x09;
const CMD_VIAL_QMK_SETTINGS_GET: u8 = 0x0A;
const CMD_VIAL_QMK_SETTINGS_SET: u8 = 0x0B;
const CMD_VIAL_QMK_SETTINGS_RESET: u8 = 0x0C;

const CMD_VIA_MACRO_GET_COUNT: u8 = 0x0C;
const CMD_VIA_MACRO_GET_BUFFER_SIZE: u8 = 0x0D;
const CMD_VIA_MACRO_GET_BUFFER: u8 = 0x0E;
const CMD_VIA_MACRO_SET_BUFFER: u8 = 0x0F;

const CMD_VIAL_GET_UNLOCK_STATUS: u8 = 0x05;
const CMD_VIAL_UNLOCK_START: u8 = 0x06;
const CMD_VIAL_UNLOCK_POLL: u8 = 0x07;

const CMD_VIA_LIGHTING_GET_VALUE: u8 = 0x08;
const VIALRGB_GET_INFO: u8 = 0x40;
const VIALRGB_GET_SUPPORTED: u8 = 0x42;

const BUFFER_FETCH_CHUNK: u8 = 28;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("ViaUnhandledError")]
    ViaUnhandledError,
    #[error("HidError {0}")]
    HidError(#[from] HidError),
    #[error("LzmaError {0}")]
    LzmaError(#[from] LzmaError),
    #[error("UTF8Error {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("JsonError {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Error {0}")]
    GeneralError(String),
}

pub fn send(device: &HidDevice, data: &[u8]) -> HidResult<usize> {
    let mut buff: [u8; MESSAGE_LENGTH + 1] = [0u8; MESSAGE_LENGTH + 1];
    for i in 0..data.len() {
        buff[i + 1] = data[i];
    }
    device.write(&buff)
}

pub fn recv(device: &HidDevice) -> HidResult<[u8; MESSAGE_LENGTH]> {
    let mut buff: [u8; MESSAGE_LENGTH] = [0u8; MESSAGE_LENGTH];
    match device.read_timeout(&mut buff, 500) {
        Ok(_size) => Ok(buff),
        Err(e) => Err(e),
    }
}

pub fn send_recv(
    device: &HidDevice,
    data_out: &[u8],
) -> Result<[u8; MESSAGE_LENGTH], Box<dyn std::error::Error>> {
    let mut attempts = 5;
    loop {
        match send(&device, &data_out) {
            Ok(_) => {
                // nothing here
            }
            Err(e) => {
                //return Err(ProtocolError::HidError(e));
                return Err(e.into());
            }
        }
        match recv(&device) {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                attempts -= 1;
                println!("hid recv error {:?}, {:?} attempts remaining", &e, attempts);
                if attempts == 0 {
                    //return Err(ProtocolError::HidError(e));
                    return Err(e.into());
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Capabilities {
    pub via_version: u8,
    pub vial_version: u32,
    pub companion_hid_version: u8,
    pub layer_count: u8,
    pub tap_dance_count: u8,
    pub combo_count: u8,
    pub key_override_count: u8,
    pub alt_repeat_key_count: u8,
    pub macro_count: u8,
    pub macro_buffer_size: u16,
    pub caps_word: bool,
    pub layer_lock: bool,
}

pub fn scan_capabilities(device: &HidDevice) -> Result<Capabilities, Box<dyn std::error::Error>> {
    let via_version;
    let vial_version;
    let companion_hid_version;
    let layer_count;
    let macro_count;
    let macro_buffer_size;

    via_version = send_recv(&device, &[CMD_VIA_GET_PROTOCOL_VERSION])?[2];
    match send_recv(&device, &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_KEYBOARD_ID]) {
        Ok(buff) => {
            if buff[0] != VIA_UNHANDLED {
                vial_version = ((buff[3] as u32) << 24)
                    + ((buff[2] as u32) << 16)
                    + ((buff[1] as u32) << 8)
                    + buff[0] as u32
            } else {
                vial_version = 0
            }
        }
        Err(e) => return Err(e),
    }
    match send_recv(&device, &[HID_LAYERS_IN, GET_VERSION]) {
        Ok(buff) => {
            if buff[0] == HID_LAYERS_OUT_VERSION {
                companion_hid_version = buff[1]
            } else {
                companion_hid_version = 0
            }
        }
        Err(e) => return Err(e),
    }
    if via_version == 0 {
        layer_count = 0;
    } else {
        match send_recv(&device, &[CMD_VIA_GET_LAYER_COUNT]) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    layer_count = buff[1]
                } else {
                    layer_count = 0
                }
            }
            Err(e) => return Err(e),
        }
    }

    match send_recv(&device, &[CMD_VIA_MACRO_GET_COUNT]) {
        Ok(buff) => {
            if buff[0] != VIA_UNHANDLED {
                macro_count = buff[1]
            } else {
                macro_count = 0
            }
        }
        Err(e) => return Err(e),
    }

    match send_recv(&device, &[CMD_VIA_MACRO_GET_BUFFER_SIZE]) {
        Ok(buff) => {
            if buff[0] != VIA_UNHANDLED {
                macro_buffer_size = ((buff[1] as u16) << 8) + (buff[2] as u16);
            } else {
                macro_buffer_size = 0
            }
        }
        Err(e) => return Err(e),
    }

    if vial_version < VIAL_PROTOCOL_DYNAMIC {
        return Ok(Capabilities {
            via_version: via_version,
            vial_version: vial_version,
            companion_hid_version: companion_hid_version,
            layer_count: layer_count,
            macro_count: macro_count,
            macro_buffer_size: macro_buffer_size,
            tap_dance_count: 0,
            combo_count: 0,
            key_override_count: 0,
            alt_repeat_key_count: 0,
            caps_word: false,
            layer_lock: false,
        });
    }

    match send_recv(
        &device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_GET_NUMBER_OF_ENTRIES,
        ],
    ) {
        Ok(buff) => {
            if buff[0] != VIA_UNHANDLED {
                Ok(Capabilities {
                    via_version: via_version,
                    vial_version: vial_version,
                    companion_hid_version: companion_hid_version,
                    layer_count: layer_count,
                    macro_count: macro_count,
                    macro_buffer_size: macro_buffer_size,
                    tap_dance_count: buff[0],
                    combo_count: buff[1],
                    key_override_count: buff[2],
                    alt_repeat_key_count: buff[3],
                    caps_word: buff[31] & 1 != 0,
                    layer_lock: buff[31] & 2 != 0,
                })
            } else {
                Err(ProtocolError::ViaUnhandledError.into())
            }
        }
        Err(e) => Err(e),
    }
}

pub fn load_vial_meta(device: &HidDevice) -> Result<Value, Box<dyn std::error::Error>> {
    let meta_size: u32;
    let mut block: u32;
    let mut remaining_size: i64;
    match send_recv(&device, &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_SIZE]) {
        Ok(buff) => {
            if buff[0] != VIA_UNHANDLED {
                meta_size = ((buff[3] as u32) << 24)
                    + ((buff[2] as u32) << 16)
                    + ((buff[1] as u32) << 8)
                    + buff[0] as u32;
            } else {
                return Err(ProtocolError::ViaUnhandledError.into());
            }
        }
        Err(e) => return Err(e.into()),
    }
    remaining_size = meta_size as i64;
    block = 0;
    let mut raw_meta = Vec::new();
    while remaining_size > 0 {
        let block1 = (block >> 24 & 0xFF) as u8;
        let block2 = (block >> 16 & 0xFF) as u8;
        let block3 = (block >> 8 & 0xFF) as u8;
        let block4 = (block & 0xFF) as u8;
        match send_recv(
            &device,
            &[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_GET_DEFINITION,
                block4,
                block3,
                block2,
                block1,
            ],
        ) {
            Ok(buff) => {
                if remaining_size >= MESSAGE_LENGTH as i64 {
                    raw_meta.extend_from_slice(&buff);
                    remaining_size = remaining_size - MESSAGE_LENGTH as i64;
                } else {
                    raw_meta.extend_from_slice(&buff[0..remaining_size as usize]);
                    remaining_size = 0;
                }
            }
            Err(e) => return Err(e),
        }
        block = block + 1;
    }
    let meta_str = String::from_utf8(lzma::decompress(&raw_meta)?)?;
    //println!("{}", meta_str);
    let meta: Value = serde_json::from_str(&meta_str)?;
    Ok(meta)
}

#[derive(Debug)]
pub struct Keymap {
    rows: u8,
    cols: u8,
    layers: u8,
    keys: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Error, Debug)]
#[error("{0}")]
pub struct KeymapError(String);

impl Keymap {
    pub fn from_json(
        rows: u8,
        cols: u8,
        layers: u8,
        layers_data: &Vec<Value>,
    ) -> Result<Keymap, Box<dyn std::error::Error>> {
        let mut keys = Vec::<u8>::new();
        for layer in layers_data {
            for row in layer
                .as_array()
                .ok_or("layer content should be array of rows")?
            {
                for value in row
                    .as_array()
                    .ok_or("row content should be array of keycodes")?
                {
                    let keycode: u16 = match value {
                        Value::Number(_) => 0,
                        Value::String(value) => {
                            if value.starts_with("0x") {
                                let (_, hex) =
                                    value.split_once("x").ok_or("Incorrect hex encoding")?;
                                u16::from_str_radix(hex, 16)?
                            } else {
                                keycodes::name_to_qid(&value)?
                            }
                        }
                        _ => {
                            return Err(KeymapError(
                                "keycode should be number or string".to_string(),
                            )
                            .into());
                        }
                    };
                    keys.push((keycode >> 8) as u8);
                    keys.push((keycode & 0xFF) as u8);
                }
            }
        }
        Ok(Keymap {
            rows,
            cols,
            layers,
            keys,
        })
    }

    pub fn to_json(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut result = Vec::new();
        for layer_num in 0..self.layers {
            let mut layer = Vec::new();
            for row_num in 0..self.rows {
                let mut row = Vec::new();
                for col_num in 0..self.cols {
                    row.push(Value::String(self.get_long(layer_num, row_num, col_num)?));
                }
                layer.push(Value::Array(row));
            }
            result.push(Value::Array(layer));
        }
        Ok(Value::Array(result))
    }

    pub fn get_short(
        &self,
        layer: u8,
        row: u8,
        col: u8,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if layer >= self.layers {
            Err(KeymapError("non existing layer requested".to_string()).into())
        } else if row >= self.rows {
            Err(KeymapError("non existing row requested".to_string()).into())
        } else if col >= self.cols {
            Err(KeymapError("non existing col requested".to_string()).into())
        } else {
            let offset = (layer as usize * self.rows as usize * self.cols as usize * 2)
                + (row as usize * self.cols as usize * 2)
                + (col as usize * 2);
            let v1 = self.keys[offset];
            let v2 = self.keys[offset + 1];
            let kk = ((v1 as u16) << 8) + (v2 as u16);
            Ok(keycodes::qid_to_short(kk))
        }
    }

    pub fn get_long(
        &self,
        layer: u8,
        row: u8,
        col: u8,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if layer >= self.layers {
            Err(KeymapError("non existing layer requested".to_string()).into())
        } else if row >= self.rows {
            Err(KeymapError("non existing row requested".to_string()).into())
        } else if col >= self.cols {
            Err(KeymapError("non existing col requested".to_string()).into())
        } else {
            let offset = (layer as usize * self.rows as usize * self.cols as usize * 2)
                + (row as usize * self.cols as usize * 2)
                + (col as usize * 2);
            let v1 = self.keys[offset];
            let v2 = self.keys[offset + 1];
            let kk = ((v1 as u16) << 8) + (v2 as u16);
            Ok(keycodes::qid_to_name(kk))
        }
    }
}

pub fn load_layers_keys(
    device: &HidDevice,
    layers: u8,
    rows: u8,
    cols: u8,
) -> Result<Keymap, Box<dyn std::error::Error>> {
    let size: u16 = layers as u16 * rows as u16 * cols as u16 * 2;
    let mut keys = Vec::new();
    let mut offset: u16 = 0;
    while offset < size {
        let read_size: u8 = min(size - offset, BUFFER_FETCH_CHUNK as u16) as u8;
        let offset1 = ((offset >> 8) & 0xFF) as u8;
        let offset2 = (offset & 0xFF) as u8;
        match send_recv(
            &device,
            &[CMD_VIA_KEYMAP_GET_BUFFER, offset1, offset2, read_size],
        ) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    //println!("{:?}", &buff[4..(read_size + 4) as usize]);
                    keys.extend_from_slice(&buff[4..(read_size + 4) as usize]);
                } else {
                    println!("UNHANDLED");
                }
            }
            Err(e) => return Err(e.into()),
        }
        offset += read_size as u16;
    }
    // println!("llen {:?}, {:?}", keys.len(), size);
    Ok(Keymap {
        layers: layers,
        rows: rows,
        cols: cols,
        keys: keys,
    })
}

pub fn set_keycode(
    device: &HidDevice,
    layer: u8,
    row: u8,
    col: u8,
    keycode: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let kk1 = ((keycode >> 8) & 0xFF) as u8;
    let kk2 = (keycode & 0xFF) as u8;
    match send(&device, &[CMD_VIA_SET_KEYCODE, layer, row, col, kk1, kk2]) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::HidError(e).into()),
    }
}

#[derive(Debug)]
pub struct LockedStatus {
    pub locked: bool,
    pub unlock_in_progress: bool,
    pub unlock_buttons: Vec<(u8, u8)>,
}

pub fn get_locked_status(device: &HidDevice) -> Result<LockedStatus, Box<dyn std::error::Error>> {
    match send_recv(&device, &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_UNLOCK_STATUS]) {
        Ok(data) => {
            // println!("{:?}", data);
            let mut unlock_buttons = Vec::new();
            let locked = data[0] == 0;
            let unlock_in_progress = data[1] == 1;
            for i in 0..15 {
                let row = data[2 + i * 2];
                let col = data[3 + i * 2];
                if row != 255 && col != 255 {
                    unlock_buttons.push((row, col))
                }
            }
            Ok(LockedStatus {
                locked,
                unlock_in_progress,
                unlock_buttons,
            })
        }
        Err(e) => Err(e.into()),
    }
}

pub fn start_unlock(device: &HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    match send_recv(&device, &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_UNLOCK_START]) {
        Ok(_) => {
            //println!("start_unlock {:?}", data);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn unlock_poll(device: &HidDevice) -> Result<(bool, u8), Box<dyn std::error::Error>> {
    match send_recv(&device, &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_UNLOCK_POLL]) {
        Ok(data) => {
            //println!("unlock poll{:?}", data);
            let unlocked = data[0] == 1;
            let seconds_remaining = data[2];
            Ok((unlocked, seconds_remaining))
        }
        Err(e) => Err(e.into()),
    }
}

pub fn load_uid(device: &HidDevice) -> Result<u64, Box<dyn std::error::Error>> {
    match send_recv(&device, &[CMD_VIA_VIAL_PREFIX]) {
        Ok(data) => {
            let mut uid_bytes: [u8; 8] = [0; 8];
            uid_bytes.copy_from_slice(&data[4..12]);
            let uid: u64 = u64::from_le_bytes(uid_bytes);
            Ok(uid)
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug)]
pub struct RGBInfo {
    pub version: u16,
    pub max_brightness: u8,
    pub effects: Vec<u16>,
}

impl RGBInfo {
    pub fn effect_name(id: u16) -> Result<&'static str, Box<dyn std::error::Error>> {
        match id {
            0 => Ok("Disable"),
            1 => Ok("Direct Control"),
            2 => Ok("Solid Color"),
            3 => Ok("Alphas Mods"),
            4 => Ok("Gradient Up Down"),
            5 => Ok("Gradient Left Right"),
            6 => Ok("Breathing"),
            7 => Ok("Band Sat"),
            8 => Ok("Band Val"),
            9 => Ok("Band Pinwheel Sat"),
            10 => Ok("Band Pinwheel Val"),
            11 => Ok("Band Spiral Sat"),
            12 => Ok("Band Spiral Val"),
            13 => Ok("Cycle All"),
            14 => Ok("Cycle Left Right"),
            15 => Ok("Cycle Up Down"),
            16 => Ok("Rainbow Moving Chevron"),
            17 => Ok("Cycle Out In"),
            18 => Ok("Cycle Out In Dual"),
            19 => Ok("Cycle Pinwheel"),
            20 => Ok("Cycle Spiral"),
            21 => Ok("Dual Beacon"),
            22 => Ok("Rainbow Beacon"),
            23 => Ok("Rainbow Pinwheels"),
            24 => Ok("Raindrops"),
            25 => Ok("Jellybean Raindrops"),
            26 => Ok("Hue Breathing"),
            27 => Ok("Hue Pendulum"),
            28 => Ok("Hue Wave"),
            29 => Ok("Typing Heatmap"),
            30 => Ok("Digital Rain"),
            31 => Ok("Solid Reactive Simple"),
            32 => Ok("Solid Reactive"),
            33 => Ok("Solid Reactive Wide"),
            34 => Ok("Solid Reactive Multiwide"),
            35 => Ok("Solid Reactive Cross"),
            36 => Ok("Solid Reactive Multicross"),
            37 => Ok("Solid Reactive Nexus"),
            38 => Ok("Solid Reactive Multinexus"),
            39 => Ok("Splash"),
            40 => Ok("Multisplash"),
            41 => Ok("Solid Splash"),
            42 => Ok("Solid Multisplash"),
            43 => Ok("Pixel Rain"),
            44 => Ok("Pixel Fractal"),
            _ => Err("no such effect".into()),
        }
    }
}

impl fmt::Display for RGBInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "RGB verions: {}, max_brightness: {}\n", self.version, self.max_brightness)?;
        write!(f, "supported_effects:\n")?;
        for effect in &self.effects {
            match  RGBInfo::effect_name(*effect) {
                 Ok(name) => {
                     write!(f, "\t{}) {}\n", effect, name)?;
                 }
                 Err(_) => {}
            };
        }
        Ok(())
    }
}


pub fn load_rgb_info(device: &HidDevice) -> Result<RGBInfo, Box<dyn std::error::Error>> {
    let version: u16;
    let max_brightness: u8;
    let mut effects: Vec<u16> = Vec::new();
    effects.push(0);
    match send_recv(&device, &[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_INFO]) {
        Ok(data) => {
            if data[0] != VIA_UNHANDLED {
                version = (data[2] as u16) + ((data[3] as u16) << 8);
                max_brightness = data[4];
                let mut effect: u16 = 0;
                'top: loop {
                    let e2 = (effect >> 8 & 0xFF) as u8;
                    let e1 = (effect & 0xFF) as u8;

                    match send_recv(
                        &device,
                        &[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_SUPPORTED, e1, e2],
                    ) {
                        Ok(data) => {
                            for i in 0..15 {
                                effect = (data[i * 2 + 2] as u16) + ((data[i * 2 + 3] as u16) << 8);
                                if effect == 0xFFFF {
                                    break 'top;
                                }
                                effects.push(effect);
                            }
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            } else {
                version = 0;
                max_brightness = 0;
            }
        }
        Err(e) => return Err(e.into()),
    }
    Ok(RGBInfo {
        version,
        max_brightness,
        effects,
    })
}
