use crate::keycodes;
use crate::protocol::{
    CMD_VIA_VIAL_PREFIX, CMD_VIAL_DYNAMIC_ENTRY_OP, DYNAMIC_VIAL_COMBO_GET, DYNAMIC_VIAL_COMBO_SET,
    ProtocolError, VIA_UNHANDLED, send, send_recv,
};
use hidapi::HidDevice;
use serde_json::{Value, json};
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
    pub fn from_string(
        index: u8,
        value: &str,
        vial_version: u32,
    ) -> Result<Combo, Box<dyn std::error::Error>> {
        let (keys_all, output) = value
            .split_once("=")
            .ok_or("resulting action should be declared after =")?;
        let keys: Vec<_> = keys_all.split("+").collect();
        let mut ks: [u16; 4] = [0x0; 4];
        let out = keycodes::name_to_qid(output, vial_version)?;
        for (idx, kn) in keys.iter().enumerate() {
            ks[idx] = keycodes::name_to_qid(kn, vial_version)?;
        }
        Ok(Combo {
            index,
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

    pub fn from_json(
        index: u8,
        combo_json: &Value,
        vial_version: u32,
    ) -> Result<Combo, Box<dyn std::error::Error>> {
        let mut ks: [u16; 5] = [0x0; 5];
        let values = combo_json
            .as_array()
            .ok_or("Combo should be encoded into array")?;
        for (pos, val) in values.iter().enumerate() {
            let value_string = val.as_str().ok_or("Combo elements should be strings")?;
            let qid = keycodes::name_to_qid(value_string, vial_version)?;
            match pos {
                0..=4 => ks[pos] = qid,
                _ => {
                    return Err(ComboFormatError(
                        "combo array should be strictly 5 elements long".to_string(),
                    )
                    .into());
                }
            }
        }
        Ok(Combo {
            index,
            key1: ks[0],
            key2: ks[1],
            key3: ks[2],
            key4: ks[3],
            output: ks[4],
        })
    }

    pub fn dump(&self, vial_version: u32) -> Result<(), std::fmt::Error> {
        print!("{}) ", self.index);
        if self.is_empty() {
            print!("EMPTY");
        } else {
            if self.key1 != 0 {
                print!("{}", keycodes::qid_to_name(self.key1, vial_version));
            }
            if self.key2 != 0 {
                print!(" + {}", keycodes::qid_to_name(self.key2, vial_version));
            }
            if self.key3 != 0 {
                print!(" + {}", keycodes::qid_to_name(self.key3, vial_version));
            }
            if self.key4 != 0 {
                print!(" + {}", keycodes::qid_to_name(self.key4, vial_version));
            }
            print!(" = {}", keycodes::qid_to_name(self.output, vial_version));
        }
        Ok(())
    }

}

pub fn load_combos(
    device: &HidDevice,
    count: u8,
) -> Result<Vec<Combo>, Box<dyn std::error::Error>> {
    let mut combos: Vec<Combo> = vec![];
    for idx in 0..count {
        match send_recv(
            device,
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
            Err(e) => return Err(e),
        }
    }
    Ok(combos)
}

pub fn load_combos_from_json(
    combos_json: &Value,
    vial_version: u32,
) -> Result<Vec<Combo>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let combos = combos_json
        .as_array()
        .ok_or("Combos should be encoded as array of arrays")?;
    for (i, combo) in combos.iter().enumerate() {
        result.push(Combo::from_json(i as u8, combo, vial_version)?)
    }
    Ok(result)
}

pub fn set_combo(device: &HidDevice, combo: &Combo) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        device,
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

pub fn combos_to_json(
    combos: &Vec<Combo>,
    vial_version: u32,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for combo in combos {
        result.push(json!([
            keycodes::qid_to_name(combo.key1, vial_version),
            keycodes::qid_to_name(combo.key2, vial_version),
            keycodes::qid_to_name(combo.key3, vial_version),
            keycodes::qid_to_name(combo.key4, vial_version),
            keycodes::qid_to_name(combo.output, vial_version),
        ]))
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_two_buttons() {
        let combo = Combo::from_string(0, &"KC_V + KC_B = KC_Z".to_string(), 6).unwrap();
        assert_eq!(combo.index, 0);
        assert_eq!(keycodes::qid_to_name(combo.key1, 6), "KC_V");
        assert_eq!(keycodes::qid_to_name(combo.key2, 6), "KC_B");
        assert_eq!(combo.key3, 0);
        assert_eq!(combo.key4, 0);
        assert_eq!(keycodes::qid_to_name(combo.output, 6), "KC_Z");
    }

    #[test]
    fn test_from_string_one_button() {
        let combo = Combo::from_string(0, &"KC_A = KC_B".to_string(), 6).unwrap();
        assert_eq!(keycodes::qid_to_name(combo.key1, 6), "KC_A");
        assert_eq!(combo.key2, 0);
        assert_eq!(keycodes::qid_to_name(combo.output, 6), "KC_B");
    }

    #[test]
    fn test_from_string_three_buttons() {
        let combo = Combo::from_string(0, &"KC_A + KC_B + KC_C = KC_D".to_string(), 6).unwrap();
        assert_eq!(keycodes::qid_to_name(combo.key1, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(combo.key2, 6), "KC_B");
        assert_eq!(keycodes::qid_to_name(combo.key3, 6), "KC_C");
        assert_eq!(combo.key4, 0);
        assert_eq!(keycodes::qid_to_name(combo.output, 6), "KC_D");
    }

    #[test]
    fn test_from_string_four_buttons() {
        let combo =
            Combo::from_string(0, &"KC_A + KC_B + KC_C + KC_D = KC_E".to_string(), 6).unwrap();
        assert_eq!(keycodes::qid_to_name(combo.key1, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(combo.key2, 6), "KC_B");
        assert_eq!(keycodes::qid_to_name(combo.key3, 6), "KC_C");
        assert_eq!(keycodes::qid_to_name(combo.key4, 6), "KC_D");
        assert_eq!(keycodes::qid_to_name(combo.output, 6), "KC_E");
    }

    #[test]
    fn test_from_string_invalid_format() {
        let result = Combo::from_string(0, &"KC_A + KC_B KC_Z".to_string(), 6);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "resulting action should be declared after ="
        );
    }

    #[test]
    fn test_from_string_invalid_keycode() {
        let result = Combo::from_string(0, &"KC_A + INVALID_KEY = KC_Z".to_string(), 6);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_combo() {
        let combo = Combo::empty(5);
        assert_eq!(combo.index, 5);
        assert_eq!(combo.key1, 0);
        assert_eq!(combo.key2, 0);
        assert_eq!(combo.key3, 0);
        assert_eq!(combo.key4, 0);
        assert_eq!(combo.output, 0);
        assert!(combo.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let empty_combo = Combo::empty(0);
        assert!(empty_combo.is_empty());

        let non_empty_combo = Combo::from_string(0, &"KC_A = KC_B".to_string(), 6).unwrap();
        assert!(!non_empty_combo.is_empty());
    }

    #[test]
    fn test_from_json_valid() {
        let combo_json = json!(["KC_A", "KC_B", "KC_C", "KC_D", "KC_E"]);
        let combo = Combo::from_json(0, &combo_json, 6).unwrap();
        assert_eq!(keycodes::qid_to_name(combo.key1, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(combo.key2, 6), "KC_B");
        assert_eq!(keycodes::qid_to_name(combo.key3, 6), "KC_C");
        assert_eq!(keycodes::qid_to_name(combo.key4, 6), "KC_D");
        assert_eq!(keycodes::qid_to_name(combo.output, 6), "KC_E");

        let combo_json_less_keys = json!(["KC_A", "KC_NO", "KC_NO", "KC_NO", "KC_B"]);
        let combo_less = Combo::from_json(0, &combo_json_less_keys, 6).unwrap();
        assert_eq!(keycodes::qid_to_name(combo_less.key1, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(combo_less.output, 6), "KC_B");
    }

    #[test]
    fn test_from_json_invalid_length_too_long() {
        let combo_json = json!(["KC_A", "KC_B", "KC_C", "KC_D", "KC_E", "KC_F"]); // 6 elements
        let result = Combo::from_json(0, &combo_json, 6);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "combo array should be strictly 5 elements long"
        );
    }

    #[test]
    fn test_from_json_non_string_elements() {
        let combo_json = json!(["KC_A", 123, "KC_C", "KC_D", "KC_E"]);
        let result = Combo::from_json(0, &combo_json, 6);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Combo elements should be strings"
        );
    }

    #[test]
    fn test_combos_to_json_empty() {
        let combos = vec![];
        let json_values = combos_to_json(&combos, 6).unwrap();
        assert!(json_values.is_empty());
    }

    #[test]
    fn test_combos_to_json_single_combo() {
        let combo = Combo::from_string(0, &"KC_V = KC_Z".to_string(), 6).unwrap();
        let combos = vec![combo];
        let json_values = combos_to_json(&combos, 6).unwrap();
        assert_eq!(json_values.len(), 1);
        assert_eq!(
            json_values[0],
            json!(["KC_V", "KC_NO", "KC_NO", "KC_NO", "KC_Z"])
        );
    }

    #[test]
    fn test_combos_to_json_multiple_combos() {
        let combo1 = Combo::from_string(0, &"KC_A = KC_B".to_string(), 6).unwrap();
        let combo2 = Combo::from_string(1, &"KC_C + KC_D = KC_E".to_string(), 6).unwrap();
        let combos = vec![combo1, combo2];
        let json_values = combos_to_json(&combos, 6).unwrap();
        assert_eq!(json_values.len(), 2);
        assert_eq!(
            json_values[0],
            json!(["KC_A", "KC_NO", "KC_NO", "KC_NO", "KC_B"])
        );
        assert_eq!(
            json_values[1],
            json!(["KC_C", "KC_D", "KC_NO", "KC_NO", "KC_E"])
        );
    }
}
