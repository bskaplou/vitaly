use crate::protocol::{
    send_recv, ProtocolError, CMD_VIAL_QMK_SETTINGS_GET, CMD_VIAL_QMK_SETTINGS_QUERY,
    CMD_VIAL_QMK_SETTINGS_RESET, CMD_VIAL_QMK_SETTINGS_SET, CMD_VIA_VIAL_PREFIX, MESSAGE_LENGTH,
};
use hidapi::HidDevice;
use serde_json::Value;
use std::cmp::max;
use std::collections::HashMap;

pub fn load_qmk_definitions() -> serde_json::Result<serde_json::Value> {
    let qmk_settings_json = include_str!("qmk_settings.json");
    let qmk_settings: serde_json::Value = serde_json::from_str(qmk_settings_json)?;
    Ok(qmk_settings)
}

pub fn load_qmk_qsids(device: &HidDevice) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    let mut cur = 0u16;
    let mut qsids = Vec::new();
    'o: loop {
        match send_recv(
            &device,
            &[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_QMK_SETTINGS_QUERY,
                (cur & 0xFF) as u8,
                ((cur >> 8) & 0xFF) as u8,
            ],
        ) {
            Ok(buff) => {
                for i in 0..(MESSAGE_LENGTH / 2) {
                    let qsid = (buff[i * 2] as u16) + ((buff[i * 2 + 1] as u16) << 8);
                    cur = max(cur, qsid);
                    if qsid == 0xFFFF {
                        break 'o;
                    }
                    qsids.push(qsid);
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(qsids)
}

#[derive(Debug, Copy, Clone)]
pub struct QmkValue {
    value: u32,
}

impl QmkValue {
    pub fn get(&self) -> u32 {
        self.value
    }

    pub fn get_bool(&self, bit: u8) -> bool {
        self.value & (1 << bit) != 0
    }
}

pub fn get_qmk_value(
    device: &HidDevice,
    qsid: u16,
    width: u8,
) -> Result<QmkValue, Box<dyn std::error::Error>> {
    match send_recv(
        &device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_QMK_SETTINGS_GET,
            (qsid & 0xFF) as u8,
            ((qsid >> 8) & 0xFF) as u8,
        ],
    ) {
        Ok(buff) => {
            if buff[0] != 0 {
                return Err(ProtocolError::ViaUnhandledError.into());
            }
            let value;
            match width {
                1 => value = buff[1] as u32,
                2 => value = (buff[1] as u32) + ((buff[2] as u32) << 8),
                4 => {
                    value = (buff[1] as u32)
                        + ((buff[2] as u32) << 8)
                        + ((buff[3] as u32) << 16)
                        + ((buff[3] as u32) << 24)
                }
                _ => value = buff[1] as u32,
            }
            Ok(QmkValue { value: value })
        }
        Err(e) => Err(e.into()),
    }
}

pub fn set_qmk_value(
    device: &HidDevice,
    qsid: u16,
    value: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let buff: [u8; 8] = [
        CMD_VIA_VIAL_PREFIX,
        CMD_VIAL_QMK_SETTINGS_SET,
        (qsid & 0xFF) as u8,
        ((qsid >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 24) & 0xFF) as u8,
    ];
    match send_recv(&device, &buff) {
        Ok(buff) => {
            if buff[0] != 0 {
                return Err(ProtocolError::GeneralError(
                    "Unexpected protocol response".to_string(),
                )
                .into());
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn reset_qmk_values(device: &HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    let buff: [u8; 2] = [CMD_VIA_VIAL_PREFIX, CMD_VIAL_QMK_SETTINGS_RESET];
    match send_recv(&device, &buff) {
        Ok(buff) => {
            if buff[0] != 0 {
                return Err(ProtocolError::GeneralError(
                    "Unexpected protocol response".to_string(),
                )
                .into());
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn load_qmk_settings_from_json(
    settings_json: &Value,
) -> Result<HashMap<u16, QmkValue>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();
    let settings = settings_json
        .as_object()
        .ok_or("Settings should be an object")?;
    for (key, value) in settings {
        let qsid: u16 = key.parse()?;
        let val = value.as_u64().ok_or("value shoudld be u32")? as u32;
        result.insert(qsid, QmkValue { value: val });
    }
    Ok(result)
}
