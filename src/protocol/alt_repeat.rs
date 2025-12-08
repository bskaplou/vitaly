use crate::keycodes;
use crate::protocol::{
    CMD_VIA_VIAL_PREFIX, CMD_VIAL_DYNAMIC_ENTRY_OP, DYNAMIC_VIAL_ALT_REPEAT_KEY_GET,
    DYNAMIC_VIAL_ALT_REPEAT_KEY_SET, ProtocolError, VIA_UNHANDLED, send, send_recv,
};
use hidapi::HidDevice;
use serde_json::{Value, json};
use std::fmt;

#[derive(Debug)]
pub struct AltRepeat {
    pub index: u8,
    pub keycode: u16,
    pub alt_keycode: u16,
    pub allowed_mods: u8,
    pub arep_option_default_to_this_alt_key: bool,
    pub arep_option_bidirectional: bool,
    pub arep_option_ignore_mod_handedness: bool,
    pub arep_enabled: bool,
}

impl AltRepeat {
    pub fn options(&self) -> u8 {
        let mut options = 0u8;
        if self.arep_option_default_to_this_alt_key {
            options |= 1;
        }
        if self.arep_option_bidirectional {
            options |= 1 << 1;
        }
        if self.arep_option_ignore_mod_handedness {
            options |= 1 << 2;
        }
        if self.arep_enabled {
            options |= 1 << 3;
        }
        options
    }

    pub fn from_string(index: u8, value: &str, vial_version: u32) -> Result<AltRepeat, Box<dyn std::error::Error>> {
        let cleaned = value.replace(" ", "");
        let keys: Vec<_> = cleaned.split(";").filter(|k| !k.is_empty()).collect();

        let mut keycode = 0u16;
        let mut alt_keycode = 0u16;
        let mut allowed_mods = 0u8;
        let mut arep_option_default_to_this_alt_key = false;
        let mut arep_option_bidirectional = false;
        let mut arep_option_ignore_mod_handedness = false;
        let mut arep_enabled = false;

        if !keys.is_empty() {
            for part in keys {
                let (left, right) = part.split_once("=").ok_or("each part should contain =")?;
                match left {
                    "keycode" | "k" => keycode = keycodes::name_to_qid(right, vial_version)?,
                    "alt_keycode" | "a" => alt_keycode = keycodes::name_to_qid(right, vial_version)?,
                    "allowed_mods" | "m" => allowed_mods = keycodes::name_to_bitmod(right)?,
                    "options" | "option" | "opt" | "o" => {
                        for o in right.split("|") {
                            match o {
                                "arep_option_default_to_this_alt_key"
                                | "option_default_to_this_alt_key"
                                | "default_to_this_alt_key" => {
                                    arep_option_default_to_this_alt_key = true
                                }
                                "arep_option_bidirectional"
                                | "option_bidirectional"
                                | "bidirectional" => arep_option_bidirectional = true,
                                "arep_option_ignore_mod_handedness"
                                | "option_ignore_mod_handedness"
                                | "ignore_mod_handedness" => {
                                    arep_option_ignore_mod_handedness = true
                                }
                                "arep_enabled" | "enabled" => arep_enabled = true,
                                _ => {
                                    return Err(keycodes::KeyParsingError(
                                        format!("Unknown option {}", left).to_string(),
                                    )
                                    .into());
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(keycodes::KeyParsingError(
                            format!("Unknown setting {}", left).to_string(),
                        )
                        .into());
                    }
                }
            }
        }
        Ok(AltRepeat {
            index,
            keycode,
            alt_keycode,
            allowed_mods,
            arep_option_default_to_this_alt_key,
            arep_option_bidirectional,
            arep_option_ignore_mod_handedness,
            arep_enabled,
        })
    }

    pub fn from_json(
        index: u8,
        alt_repeat_json: &Value,
        vial_version: u32,
    ) -> Result<AltRepeat, Box<dyn std::error::Error>> {
        let mut keycode = 0u16;
        let mut alt_keycode = 0u16;
        let mut allowed_mods = 0u8;
        let mut arep_option_default_to_this_alt_key = false;
        let mut arep_option_bidirectional = false;
        let mut arep_option_ignore_mod_handedness = false;
        let mut arep_enabled = false;
        let alt_repeat = alt_repeat_json
            .as_object()
            .ok_or("alt_repeat element should be an object")?;

        for (key, value) in alt_repeat {
            match key.as_str() {
                "keycode" => {
                    keycode = keycodes::name_to_qid(
                        value.as_str().ok_or("keycode value should be string")?,
                        vial_version,
                    )?;
                }
                "alt_keycode" => {
                    alt_keycode = keycodes::name_to_qid(
                        value.as_str().ok_or("keycode value should be string")?,
                        vial_version,
                    )?;
                }
                "allowed_mods" => {
                    allowed_mods = value
                        .as_u64()
                        .ok_or("allowed_mods value should be a number")?
                        as u8;
                }
                "options" => {
                    let options = value.as_u64().ok_or("options value should be a number")? as u16;
                    arep_option_default_to_this_alt_key = options & (1 << 0) == (1 << 0);
                    arep_option_bidirectional = options & (1 << 1) == (1 << 1);
                    arep_option_ignore_mod_handedness = options & (1 << 2) == (1 << 2);
                    arep_enabled = options & (1 << 3) == (1 << 3);
                }
                _ => {
                    return Err(keycodes::KeyParsingError(
                        format!("Unknown alt_repeat key {}", key).to_string(),
                    )
                    .into());
                }
            }
        }

        Ok(AltRepeat {
            index,
            keycode,
            alt_keycode,
            allowed_mods,
            arep_option_default_to_this_alt_key,
            arep_option_bidirectional,
            arep_option_ignore_mod_handedness,
            arep_enabled,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.keycode == 0 && !self.arep_enabled
    }

    pub fn empty(index: u8) -> AltRepeat {
        AltRepeat {
            index,
            keycode: 0,
            alt_keycode: 0,
            allowed_mods: 0,
            arep_option_default_to_this_alt_key: false,
            arep_option_bidirectional: false,
            arep_option_ignore_mod_handedness: false,
            arep_enabled: false,
        }
    }
}

impl fmt::Display for AltRepeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.is_empty() {
            Ok(write!(f, "EMPTY")?)
        } else {
            write!(f, "keycode = {}; ", keycodes::qid_to_name(self.keycode, 6))?;
            write!(
                f,
                "alt_keycode = {}; ",
                keycodes::qid_to_name(self.alt_keycode, 6)
            )?;
            write!(
                f,
                "\n\tallowed_mods = {};",
                keycodes::bitmod_to_name(self.allowed_mods)
            )?;
            write!(
                f,
                "\n\tarep_option_default_to_this_alt_key = {}",
                self.arep_option_default_to_this_alt_key
            )?;
            write!(
                f,
                "\n\tarep_option_bidirectional = {}",
                self.arep_option_bidirectional
            )?;
            write!(
                f,
                "\n\tarep_option_ignore_mod_handedness = {}",
                self.arep_option_ignore_mod_handedness
            )?;
            Ok(write!(f, "\n\tarep_enabled = {}", self.arep_enabled)?)
        }
    }
}

pub fn load_alt_repeats(
    device: &HidDevice,
    count: u8,
) -> Result<Vec<AltRepeat>, Box<dyn std::error::Error>> {
    let mut altrepeats: Vec<AltRepeat> = vec![];
    for idx in 0..count {
        match send_recv(
            device,
            &[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_DYNAMIC_ENTRY_OP,
                DYNAMIC_VIAL_ALT_REPEAT_KEY_GET,
                idx,
            ],
        ) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    let altreapeat = AltRepeat {
                        index: idx,
                        keycode: ((buff[2] as u16) << 8) + buff[1] as u16,
                        alt_keycode: ((buff[4] as u16) << 8) + buff[3] as u16,
                        allowed_mods: buff[5],
                        arep_option_default_to_this_alt_key: buff[6] & (1 << 0) == (1 << 0),
                        arep_option_bidirectional: buff[6] & (1 << 1) == (1 << 1),
                        arep_option_ignore_mod_handedness: buff[6] & (1 << 2) == (1 << 2),
                        arep_enabled: buff[6] & (1 << 3) == (1 << 3),
                    };
                    altrepeats.push(altreapeat)
                } else {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(altrepeats)
}

pub fn load_alt_repeats_from_json(
    alt_repeats_json: &Value,
    vial_version: u32,
) -> Result<Vec<AltRepeat>, Box<dyn std::error::Error>> {
    let alt_repeats = alt_repeats_json
        .as_array()
        .ok_or("alt_repeats_json should be an array")?;
    let mut result = Vec::new();
    for (i, alt_repeat) in alt_repeats.iter().enumerate() {
        result.push(AltRepeat::from_json(i as u8, alt_repeat, vial_version)?);
    }
    Ok(result)
}

pub fn set_alt_repeat(
    device: &HidDevice,
    altrepeat: &AltRepeat,
) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_ALT_REPEAT_KEY_SET,
            altrepeat.index,
            (altrepeat.keycode & 0xFF) as u8,
            ((altrepeat.keycode >> 8) & 0xFF) as u8,
            (altrepeat.alt_keycode & 0xFF) as u8,
            ((altrepeat.alt_keycode >> 8) & 0xFF) as u8,
            altrepeat.allowed_mods,
            altrepeat.options(),
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::HidError(e).into()),
    }
}

pub fn alt_repeats_to_json(
    alt_repeats: &Vec<AltRepeat>,
    vial_version: u32,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for alt_repeat in alt_repeats {
        result.push(json!({
            "keycode": keycodes::qid_to_name(alt_repeat.keycode, vial_version),
            "alt_keycode": keycodes::qid_to_name(alt_repeat.alt_keycode, vial_version),
            "allowed_mods": alt_repeat.allowed_mods,
            "options": alt_repeat.options(),
        }))
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_from_string_two_buttons() {
        let altrepeat = AltRepeat::from_string(
            3,
            &"keycode = KC_3; alt_keycode= KC_5; options= arep_enabled;".to_string(),
            6,
        )
        .unwrap();
        assert_eq!(altrepeat.index, 3);
        assert_eq!(keycodes::qid_to_name(altrepeat.keycode, 6), "KC_3");
        assert_eq!(keycodes::qid_to_name(altrepeat.alt_keycode, 6), "KC_5");
        assert_eq!(altrepeat.allowed_mods, 0);
        assert_eq!(altrepeat.arep_enabled, true);
    }

    #[test]
    fn test_from_string_full() {
        let ar = AltRepeat::from_string(
            0,
            &"k=KC_A; a=KC_B; m=LCTL; o=enabled|bidirectional".to_string(),
            6,
        )
        .unwrap();
        assert_eq!(keycodes::qid_to_name(ar.keycode, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(ar.alt_keycode, 6), "KC_B");
        assert_eq!(ar.allowed_mods, 0b00000001); // MOD_LCTL
        assert!(ar.arep_enabled);
        assert!(ar.arep_option_bidirectional);
        assert!(!ar.arep_option_ignore_mod_handedness);
    }

    #[test]
    fn test_from_string_errors() {
        assert!(
            AltRepeat::from_string(0, &"k=KC_A; a".to_string(), 6).is_err(),
            "Missing ="
        );
        assert!(
            AltRepeat::from_string(0, &"foo=bar".to_string(), 6).is_err(),
            "Unknown key"
        );
        assert!(
            AltRepeat::from_string(0, &"o=invalid_option".to_string(), 6).is_err(),
            "Unknown option"
        );
        assert!(
            AltRepeat::from_string(0, &"k=INVALID".to_string(), 6).is_err(),
            "Invalid keycode"
        );
    }

    #[test]
    fn test_from_json_valid() {
        let json = json!({
            "keycode": "KC_A",
            "alt_keycode": "KC_B",
            "allowed_mods": 1,
            "options": 10
        });
        let ar = AltRepeat::from_json(0, &json, 6).unwrap();
        assert_eq!(keycodes::qid_to_name(ar.keycode, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(ar.alt_keycode, 6), "KC_B");
        assert_eq!(ar.allowed_mods, 1);
        assert!(!ar.arep_option_default_to_this_alt_key);
        assert!(ar.arep_option_bidirectional);
        assert!(!ar.arep_option_ignore_mod_handedness);
        assert!(ar.arep_enabled);
    }

    #[test]
    fn test_from_json_errors() {
        assert!(AltRepeat::from_json(0, &json!("not an object"), 6).is_err());
        assert!(AltRepeat::from_json(0, &json!({"keycode": 123}), 6).is_err());
        assert!(AltRepeat::from_json(0, &json!({"allowed_mods": "string"}), 6).is_err());
        assert!(AltRepeat::from_json(0, &json!({"options": "string"}), 6).is_err());
        assert!(AltRepeat::from_json(0, &json!({"unknown_key": "KC_A"}), 6).is_err());
    }

    #[test]
    fn test_options_bitmask() {
        let mut ar = AltRepeat::empty(0);
        assert_eq!(ar.options(), 0);
        ar.arep_enabled = true;
        assert_eq!(ar.options(), 8); // 1 << 3
        ar.arep_option_bidirectional = true;
        assert_eq!(ar.options(), 10); // 8 | 2
        ar.arep_option_ignore_mod_handedness = true;
        assert_eq!(ar.options(), 14); // 10 | 4
        ar.arep_option_default_to_this_alt_key = true;
        assert_eq!(ar.options(), 15); // 14 | 1
    }

    #[test]
    fn test_empty_and_is_empty() {
        let empty_ar = AltRepeat::empty(0);
        assert!(empty_ar.is_empty());

        let mut non_empty = AltRepeat::empty(1);
        non_empty.keycode = keycodes::name_to_qid(&"KC_A".to_string(), 6).unwrap();
        assert!(!non_empty.is_empty());

        let mut non_empty2 = AltRepeat::empty(2);
        non_empty2.arep_enabled = true;
        assert!(!non_empty2.is_empty());
    }

    #[test]
    fn test_display() {
        let empty_ar = AltRepeat::empty(0);
        assert_eq!(format!("{}", empty_ar), "0) EMPTY");

        let mut ar = AltRepeat::empty(1);
        ar.keycode = keycodes::name_to_qid(&"KC_A".to_string(), 6).unwrap();
        ar.alt_keycode = keycodes::name_to_qid(&"KC_B".to_string(), 6).unwrap();
        ar.arep_enabled = true;
        let display_str = format!("{}", ar);
        assert!(display_str.contains("keycode = KC_A;"));
        assert!(display_str.contains("alt_keycode = KC_B;"));
        assert!(display_str.contains("\n\tarep_enabled = true"));
    }

    #[test]
    fn test_json_round_trip() {
        let mut ar1 = AltRepeat::empty(0);
        ar1.keycode = keycodes::name_to_qid(&"KC_A".to_string(), 6).unwrap();
        ar1.arep_enabled = true;
        ar1.arep_option_bidirectional = true;

        let mut ar2 = AltRepeat::empty(1);
        ar2.keycode = keycodes::name_to_qid(&"KC_X".to_string(), 6).unwrap();
        ar2.alt_keycode = keycodes::name_to_qid(&"KC_Y".to_string(), 6).unwrap();
        ar2.allowed_mods = 1; // LCTL

        let alt_repeats = vec![ar1, ar2];

        let json_val = alt_repeats_to_json(&alt_repeats, 6).unwrap();
        let loaded_ars = load_alt_repeats_from_json(&Value::Array(json_val), 6).unwrap();

        assert_eq!(alt_repeats.len(), loaded_ars.len());
        assert_eq!(alt_repeats[0].keycode, loaded_ars[0].keycode);
        assert_eq!(alt_repeats[0].options(), loaded_ars[0].options());
        assert_eq!(alt_repeats[1].keycode, loaded_ars[1].keycode);
        assert_eq!(alt_repeats[1].alt_keycode, loaded_ars[1].alt_keycode);
        assert_eq!(alt_repeats[1].allowed_mods, loaded_ars[1].allowed_mods);
        assert_eq!(alt_repeats[1].options(), loaded_ars[1].options());
    }
}
