use crate::protocol::{
    send_recv, ProtocolError, CMD_VIAL_QMK_SETTINGS_GET, CMD_VIAL_QMK_SETTINGS_QUERY,
    CMD_VIAL_QMK_SETTINGS_RESET, CMD_VIAL_QMK_SETTINGS_SET, CMD_VIA_VIAL_PREFIX, MESSAGE_LENGTH,
};
use hidapi::HidDevice;
use serde_json::{Map, Value};
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

pub fn load_qmk_settings(
    device: &HidDevice,
) -> Result<HashMap<u16, QmkValue>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();
    let sids = load_qmk_qsids(device)?;
    let definitions = load_qmk_definitions()?;
    for group in definitions["tabs"]
        .as_array()
        .ok_or("tabs should be an array")?
    {
        for field in group["fields"]
            .as_array()
            .ok_or("fields should be an array")?
        {
            let qsid = field["qsid"].as_u64().ok_or("qsid should be a number")? as u16;
            if sids.contains(&qsid) {
                let width: u8 = match &field["width"] {
                    Value::Number(n) => n.as_u64().ok_or("width shoulbe a number")? as u8,
                    _ => 1,
                };
                let value = get_qmk_value(&device, qsid, width)?;
                result.insert(qsid, value);
            }
        }
    }
    Ok(result)
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

pub fn qmk_settings_to_json(
    values: &HashMap<u16, QmkValue>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Map::new();
    for (key, value) in values {
        result.insert(key.to_string(), value.get().into());
    }
    Ok(Value::Object(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_qmk_value_getters() {
        let qv = QmkValue { value: 5 }; // 5 in binary is 0101
        assert_eq!(qv.get(), 5);
        assert!(qv.get_bool(0)); // 1st bit is set (0-indexed)
        assert!(!qv.get_bool(1)); // 2nd bit is not set
        assert!(qv.get_bool(2)); // 3rd bit is set
        assert!(!qv.get_bool(3)); // 4th bit is not set

        let qv_zero = QmkValue { value: 0 };
        assert_eq!(qv_zero.get(), 0);
        assert!(!qv_zero.get_bool(0));
        assert!(!qv_zero.get_bool(31)); // Check highest bit for zero
    }

    #[test]
    fn test_load_qmk_definitions_ok() {
        // This test relies on qmk_settings.json being valid.
        // If the file is malformed, the build would fail before tests run.
        // So we only check for successful loading and basic structure.
        let defs = load_qmk_definitions();
        assert!(
            defs.is_ok(),
            "Failed to load qmk_definitions: {:?}",
            defs.err()
        );
        let defs_value = defs.unwrap();
        assert!(
            defs_value.is_object(),
            "QMK definitions should be a JSON object"
        );
        assert!(
            defs_value.get("tabs").is_some(),
            "QMK definitions should contain a 'tabs' key"
        );
        assert!(defs_value["tabs"].is_array(), "'tabs' should be an array");
    }

    #[test]
    fn test_load_qmk_settings_from_json_valid() {
        let json_input = json!({
            "100": 12345,
            "200": 67890
        });
        let settings = load_qmk_settings_from_json(&json_input).unwrap();
        assert_eq!(settings.len(), 2);
        assert_eq!(settings.get(&100).unwrap().get(), 12345);
        assert_eq!(settings.get(&200).unwrap().get(), 67890);

        // Test with empty object
        let empty_json = json!({});
        let empty_settings = load_qmk_settings_from_json(&empty_json).unwrap();
        assert!(empty_settings.is_empty());
    }

    #[test]
    fn test_load_qmk_settings_from_json_invalid() {
        // Invalid: not an object
        let not_an_object = json!(["100", 12345]);
        assert!(
            load_qmk_settings_from_json(&not_an_object).is_err(),
            "Should error for non-object input"
        );

        // Invalid: key not a number string
        let invalid_key = json!({
            "abc": 12345
        });
        assert!(
            load_qmk_settings_from_json(&invalid_key).is_err(),
            "Should error for non-numeric string key"
        );

        // Invalid: value not a u64
        let invalid_value = json!({
            "100": "not_a_number"
        });
        assert!(
            load_qmk_settings_from_json(&invalid_value).is_err(),
            "Should error for non-numeric value"
        );

        // Invalid: value is float
        let invalid_float_value = json!({
            "100": 123.45
        });
        assert!(
            load_qmk_settings_from_json(&invalid_float_value).is_err(),
            "Should error for float value"
        );
    }

    #[test]
    fn test_qmk_settings_to_json_valid() {
        let mut input_map = HashMap::new();
        input_map.insert(100, QmkValue { value: 12345 });
        input_map.insert(200, QmkValue { value: 67890 });

        let json_output = qmk_settings_to_json(&input_map).unwrap();
        let expected_json = json!({
            "100": 12345,
            "200": 67890
        });
        assert_eq!(json_output, expected_json);

        // Test with empty map
        let empty_map = HashMap::new();
        let empty_json_output = qmk_settings_to_json(&empty_map).unwrap();
        assert_eq!(empty_json_output, json!({}));
    }
}
