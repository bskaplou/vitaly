use crate::keycodes;
use crate::protocol::{
    CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_ENCODER, CMD_VIAL_SET_ENCODER, ProtocolError, send_recv,
};
use hidapi::HidDevice;
//use serde_json::{Value, json};
use serde_json::Value;
use std::collections::HashMap;

pub struct Encoder {
    pub index: u8,
    pub ccw: u16,
    pub cw: u16,
}

pub fn load_encoder(
    device: &HidDevice,
    layer: u8,
    index: u8,
) -> Result<Encoder, Box<dyn std::error::Error>> {
    match send_recv(
        device,
        &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_ENCODER, layer, index],
    ) {
        Ok(data) => Ok(Encoder {
            index,
            ccw: ((data[0] as u16) << 8) + (data[1] as u16),
            cw: ((data[2] as u16) << 8) + (data[3] as u16),
        }),
        Err(e) => Err(e),
    }
}

pub fn set_encoder(
    device: &HidDevice,
    layer: u8,
    index: u8,
    direction: u8,
    value: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match send_recv(
        device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_SET_ENCODER,
            layer,
            index,
            direction,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn load_encoders_from_json(
    encoders_json: &Value,
) -> Result<Vec<HashMap<u8, Encoder>>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    if matches!(encoders_json, Value::Null) {
        return Ok(result);
    }
    let layers = encoders_json
        .as_array()
        .ok_or("encoders should be encoded as array of arrays of arrays")?;
    for layer in layers {
        let mut layer_encoders = HashMap::new();
        for (idx, encoder) in layer
            .as_array()
            .ok_or("encoders should be encoded as array of arrays of arrays")?
            .iter()
            .enumerate()
        {
            let values = encoder
                .as_array()
                .ok_or("encoder values should be and array with two elements")?;
            if values.len() != 2 {
                return Err(ProtocolError::General(
                    "encoder values should be and array with two elements".to_string(),
                )
                .into());
            }
            let ccw = values[0]
                .as_str()
                .ok_or("encoder value should be a string")?;
            let ccw = keycodes::name_to_qid(ccw)?;
            let cw = values[1]
                .as_str()
                .ok_or("encoder value should be a string")?;
            let cw = keycodes::name_to_qid(cw)?;
            layer_encoders.insert(
                idx as u8,
                Encoder {
                    index: idx as u8,
                    ccw,
                    cw,
                },
            );
        }
        result.push(layer_encoders);
    }
    Ok(result)
}
