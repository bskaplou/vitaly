use hidapi::{HidDevice, HidError, HidResult};
use lzma::LzmaError;
use serde_json::Value;
use std::cmp::min;
use std::string::FromUtf8Error;
use thiserror::Error;

pub mod keycodes;

pub mod key_override;
pub use crate::protocol::key_override::{load_key_overrides, set_key_override, KeyOverride};

pub mod alt_repeat;
pub use crate::protocol::alt_repeat::{load_alt_repeats, set_alt_repeat, AltRepeat};

pub mod tap_dance;
pub use crate::protocol::tap_dance::{load_tap_dances, set_tap_dance, TapDance};

pub mod combo;
pub use crate::protocol::combo::{load_combos, set_combo, Combo};

pub mod r#macro;
pub use crate::protocol::r#macro::{load_macros, set_macros, Macro};

pub mod qmk_settings;
pub use crate::protocol::qmk_settings::{
    get_qmk_value, load_qmk_definitions, load_qmk_qsids, reset_qmk_values, set_qmk_value, QmkValue,
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
