use crate::keycodes;
use crate::protocol::{
    send, send_recv, ProtocolError, CMD_VIAL_DYNAMIC_ENTRY_OP, CMD_VIA_VIAL_PREFIX,
    DYNAMIC_VIAL_COMBO_GET, DYNAMIC_VIAL_COMBO_SET, VIA_UNHANDLED,
};
use hidapi::HidDevice;
use serde_json::Value;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct ComboFormatError(pub String);

#[derive(Debug)]
pub struct Combo {
    pub index: u8,
    pub key1: u16,
    pub key2: u16,
    pub key3: u16,
    pub key4: u16,
    pub output: u16,
}

impl Combo {
    pub fn from_strings(
        index: u8,
        keys: Vec<&str>,
        output: &str,
    ) -> Result<Combo, keycodes::KeyParsingError> {
        let mut ks: [u16; 4] = [0x0; 4];
        let out = keycodes::name_to_qid(&output.to_string())?;
        for (idx, kn) in keys.iter().enumerate() {
            ks[idx] = keycodes::name_to_qid(&kn.to_string())?;
        }
        Ok(Combo {
            index: index,
            key1: ks[0],
            key2: ks[1],
            key3: ks[2],
            key4: ks[3],
            output: out,
        })
    }

    pub fn empty(index: u8) -> Combo {
        Combo {
            index,
            key1: 0,
            key2: 0,
            key3: 0,
            key4: 0,
            output: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.output == 0 || self.key1 == 0
    }

    pub fn from_json(index: u8, combo_json: &Value) -> Result<Combo, Box<dyn std::error::Error>> {
        let mut ks: [u16; 5] = [0x0; 5];
        let values = combo_json
            .as_array()
            .ok_or("Combo should be encoded into array")?;
        for (pos, val) in values.iter().enumerate() {
            let value_string = val.as_str().ok_or("Combo elements should be strings")?;
            let qid = keycodes::name_to_qid(&value_string.to_string())?;
            match pos {
                0 | 1 | 2 | 3 | 4 => ks[pos] = qid,
                _ => {
                    return Err(ComboFormatError(
                        "combo array should be strictly 5 elements long".to_string(),
                    )
                    .into());
                }
            }
        }
        Ok(Combo {
            index: index,
            key1: ks[0],
            key2: ks[1],
            key3: ks[2],
            key4: ks[3],
            output: ks[4],
        })
    }
}

impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.is_empty() {
            Ok(write!(f, "EMPTY")?)
        } else {
            if self.key1 != 0 {
                write!(f, "{}", keycodes::qid_to_name(self.key1))?
            }
            if self.key2 != 0 {
                write!(f, " + {}", keycodes::qid_to_name(self.key2))?
            }
            if self.key3 != 0 {
                write!(f, " + {}", keycodes::qid_to_name(self.key3))?
            }
            if self.key4 != 0 {
                write!(f, " + {}", keycodes::qid_to_name(self.key4))?
            }
            Ok(write!(f, " = {}", keycodes::qid_to_name(self.output))?)
        }
    }
}

pub fn load_combos(
    device: &HidDevice,
    count: u8,
) -> Result<Vec<Combo>, Box<dyn std::error::Error>> {
    let mut combos: Vec<Combo> = vec![];
    for idx in 0..count {
        match send_recv(
            &device,
            &[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_DYNAMIC_ENTRY_OP,
                DYNAMIC_VIAL_COMBO_GET,
                idx,
            ],
        ) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    let combo = Combo {
                        index: idx,
                        key1: ((buff[2] as u16) << 8) + buff[1] as u16,
                        key2: ((buff[4] as u16) << 8) + buff[3] as u16,
                        key3: ((buff[6] as u16) << 8) + buff[5] as u16,
                        key4: ((buff[8] as u16) << 8) + buff[7] as u16,
                        output: ((buff[10] as u16) << 8) + buff[9] as u16,
                    };
                    combos.push(combo)
                } else {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(combos)
}

pub fn load_combos_from_json(
    combos_json: &Value,
) -> Result<Vec<Combo>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let combos = combos_json
        .as_array()
        .ok_or("Combos should be encoded as array of arrays")?;
    for (i, combo) in combos.iter().enumerate() {
        result.push(Combo::from_json(i as u8, &combo)?)
    }
    Ok(result)
}

pub fn set_combo(device: &HidDevice, combo: &Combo) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        &device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_COMBO_SET,
            combo.index,
            (combo.key1 & 0xFF) as u8,
            ((combo.key1 >> 8) & 0xFF) as u8,
            (combo.key2 & 0xFF) as u8,
            ((combo.key2 >> 8) & 0xFF) as u8,
            (combo.key3 & 0xFF) as u8,
            ((combo.key3 >> 8) & 0xFF) as u8,
            (combo.key4 & 0xFF) as u8,
            ((combo.key4 >> 8) & 0xFF) as u8,
            (combo.output & 0xFF) as u8,
            ((combo.output >> 8) & 0xFF) as u8,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::HidError(e).into()),
    }
}
