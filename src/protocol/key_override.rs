use crate::keycodes;
use crate::protocol::{
    CMD_VIA_VIAL_PREFIX, CMD_VIAL_DYNAMIC_ENTRY_OP, DYNAMIC_VIAL_KEY_OVERRIDE_GET,
    DYNAMIC_VIAL_KEY_OVERRIDE_SET, ProtocolError, VIA_UNHANDLED, send, send_recv,
};
use hidapi::HidDevice;
use serde_json::{Value, json};
use std::fmt;

#[derive(Debug)]
pub struct KeyOverride {
    pub index: u8,
    pub trigger: u16,
    pub replacement: u16,
    pub layers: u16,
    pub trigger_mods: u8,
    pub negative_mod_mask: u8,
    pub suppressed_mods: u8,
    pub ko_option_activation_trigger_down: bool,
    pub ko_option_activation_required_mod_down: bool,
    pub ko_option_activation_negative_mod_up: bool,
    pub ko_option_one_mod: bool,
    pub ko_option_no_reregister_trigger: bool,
    pub ko_option_no_unregister_on_other_key_down: bool,
    pub ko_enabled: bool,
}

impl KeyOverride {
    pub fn options(&self) -> u8 {
        let mut options = 0u8;
        if self.ko_option_activation_trigger_down {
            options |= 1;
        }
        if self.ko_option_activation_required_mod_down {
            options |= 1 << 1;
        }
        if self.ko_option_activation_negative_mod_up {
            options |= 1 << 2;
        }
        if self.ko_option_one_mod {
            options |= 1 << 3;
        }
        if self.ko_option_no_reregister_trigger {
            options |= 1 << 4;
        }
        if self.ko_option_no_unregister_on_other_key_down {
            options |= 1 << 5;
        }
        if self.ko_enabled {
            options |= 1 << 7;
        }
        options
    }

    pub fn from_string(index: u8, value: &str, vial_version: u32) -> Result<KeyOverride, Box<dyn std::error::Error>> {
        let spaceless = value.replace(" ", "");
        let keys: Vec<&str> = spaceless.split(";").filter(|k| !k.is_empty()).collect();

        let mut trigger = 0u16;
        let mut replacement = 0u16;
        let mut layers = 0u16;
        let mut trigger_mods = 0u8;
        let mut negative_mod_mask = 0u8;
        let mut suppressed_mods = 0u8;
        let mut ko_option_activation_trigger_down = false;
        let mut ko_option_activation_required_mod_down = false;
        let mut ko_option_activation_negative_mod_up = false;
        let mut ko_option_one_mod = false;
        let mut ko_option_no_reregister_trigger = false;
        let mut ko_option_no_unregister_on_other_key_down = false;
        let mut ko_enabled = false;

        if !keys.is_empty() {
            for part in keys {
                let (left, right) = part.split_once("=").ok_or("each part should contain =")?;
                match left {
                    "trigger" | "t" => {
                        trigger = keycodes::name_to_qid(right, vial_version)?;
                    }
                    "replacement" | "r" => replacement = keycodes::name_to_qid(right, vial_version)?,
                    "layers" | "l" => {
                        for l in right.split("|") {
                            let layer: u8 = l.parse()?;
                            layers |= 1 << layer;
                        }
                    }
                    "trigger_mods" | "tm" | "m" => trigger_mods = keycodes::name_to_bitmod(right)?,
                    "negative_mod_mask" | "nmm" | "n" => {
                        negative_mod_mask = keycodes::name_to_bitmod(right)?
                    }
                    "suppressed_mods" | "sm" | "s" => {
                        suppressed_mods = keycodes::name_to_bitmod(right)?
                    }
                    "options" | "option" | "opt" | "o" => {
                        for o in right.split("|") {
                            match o {
                                "ko_option_activation_trigger_down"
                                | "option_activation_trigger_down"
                                | "activation_trigger_down" => {
                                    ko_option_activation_trigger_down = true
                                }
                                "ko_option_activation_required_mod_down"
                                | "option_activation_required_mod_down"
                                | "activation_required_mod_down" => {
                                    ko_option_activation_required_mod_down = true
                                }
                                "ko_option_activation_negative_mod_up"
                                | "option_activation_negative_mod_up"
                                | "activation_negative_mod_up" => {
                                    ko_option_activation_negative_mod_up = true
                                }
                                "ko_option_one_mod" | "option_one_mod" | "one_mod" => {
                                    ko_option_one_mod = true
                                }
                                "ko_option_no_reregister_trigger"
                                | "option_no_reregister_trigger"
                                | "no_reregister_trigger" => ko_option_no_reregister_trigger = true,
                                "ko_option_no_unregister_on_other_key_down"
                                | "option_no_unregister_on_other_key_down"
                                | "no_unregister_on_other_key_down" => {
                                    ko_option_no_unregister_on_other_key_down = true
                                }
                                "ko_enabled" | "enabled" => ko_enabled = true,
                                _ => {
                                    return Err(keycodes::KeyParsingError(
                                        format!("Unknown option {}", o).to_string(),
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
        Ok(KeyOverride {
            index,
            trigger,
            replacement,
            layers,
            trigger_mods,
            negative_mod_mask,
            suppressed_mods,
            ko_option_activation_trigger_down,
            ko_option_activation_required_mod_down,
            ko_option_activation_negative_mod_up,
            ko_option_one_mod,
            ko_option_no_reregister_trigger,
            ko_option_no_unregister_on_other_key_down,
            ko_enabled,
        })
    }

    pub fn from_json(
        index: u8,
        key_override_json: &Value,
        vial_version: u32,
    ) -> Result<KeyOverride, Box<dyn std::error::Error>> {
        let mut trigger = 0u16;
        let mut replacement = 0u16;
        let mut layers = 0u16;
        let mut trigger_mods = 0u8;
        let mut negative_mod_mask = 0u8;
        let mut suppressed_mods = 0u8;
        let mut ko_option_activation_trigger_down = false;
        let mut ko_option_activation_required_mod_down = false;
        let mut ko_option_activation_negative_mod_up = false;
        let mut ko_option_one_mod = false;
        let mut ko_option_no_reregister_trigger = false;
        let mut ko_option_no_unregister_on_other_key_down = false;
        let mut ko_enabled = false;

        let key_override = key_override_json
            .as_object()
            .ok_or("key_override element should be an object")?;
        for (key, value) in key_override {
            match key.as_str() {
                "trigger" => {
                    trigger = keycodes::name_to_qid(
                        value.as_str().ok_or("trigger value should be string")?,
                        vial_version,
                    )?;
                }
                "replacement" => {
                    replacement = keycodes::name_to_qid(
                        value.as_str().ok_or("replacement value should be string")?,
                        vial_version,
                    )?;
                }
                "layers" => {
                    layers = value.as_u64().ok_or("layer value should be a number")? as u16;
                }
                "trigger_mods" => {
                    trigger_mods = value
                        .as_u64()
                        .ok_or("trigger_mods value should be a number")?
                        as u8;
                }
                "negative_mod_mask" => {
                    negative_mod_mask = value
                        .as_u64()
                        .ok_or("negative_mod_mask value should be a number")?
                        as u8;
                }
                "suppressed_mods" => {
                    suppressed_mods = value
                        .as_u64()
                        .ok_or("suppressed_mods value should be a number")?
                        as u8;
                }
                "options" => {
                    let options = value.as_u64().ok_or("options value should be a number")? as u16;
                    ko_option_activation_trigger_down = options & (1 << 0) == (1 << 0);
                    ko_option_activation_required_mod_down = options & (1 << 1) == (1 << 1);
                    ko_option_activation_negative_mod_up = options & (1 << 2) == (1 << 2);
                    ko_option_one_mod = options & (1 << 3) == (1 << 3);
                    ko_option_no_reregister_trigger = options & (1 << 4) == (1 << 4);
                    ko_option_no_unregister_on_other_key_down = options & (1 << 5) == (1 << 5);
                    ko_enabled = options & (1 << 7) == (1 << 7);
                }
                _ => {
                    return Err(keycodes::KeyParsingError(
                        format!("Unknown key_override key {}", key).to_string(),
                    )
                    .into());
                }
            }
        }

        Ok(KeyOverride {
            index,
            trigger,
            replacement,
            layers,
            trigger_mods,
            negative_mod_mask,
            suppressed_mods,
            ko_option_activation_trigger_down,
            ko_option_activation_required_mod_down,
            ko_option_activation_negative_mod_up,
            ko_option_one_mod,
            ko_option_no_reregister_trigger,
            ko_option_no_unregister_on_other_key_down,
            ko_enabled,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.trigger == 0 && !self.ko_enabled
    }

    pub fn empty(index: u8) -> KeyOverride {
        KeyOverride {
            index,
            trigger: 0,
            replacement: 0,
            layers: 0,
            trigger_mods: 0,
            negative_mod_mask: 0,
            suppressed_mods: 0,
            ko_option_activation_trigger_down: false,
            ko_option_activation_required_mod_down: false,
            ko_option_activation_negative_mod_up: false,
            ko_option_one_mod: false,
            ko_option_no_reregister_trigger: false,
            ko_option_no_unregister_on_other_key_down: false,
            ko_enabled: false,
        }
    }
}

impl fmt::Display for KeyOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.is_empty() {
            Ok(write!(f, "EMPTY")?)
        } else {
            write!(f, "trigger = {}; ", keycodes::qid_to_name(self.trigger, 6))?;
            write!(
                f,
                "replacement = {}; ",
                keycodes::qid_to_name(self.replacement, 6)
            )?;
            write!(f, "layers = ")?;
            let mut lne = false;
            for l in 0..16 {
                if self.layers & (1 << l) != 0 {
                    if lne {
                        write!(f, "|")?;
                    }
                    write!(f, "{}", l)?;
                    lne = true;
                }
            }
            write!(f, ";")?;
            write!(
                f,
                "\n\ttrigger_mods = {};",
                keycodes::bitmod_to_name(self.trigger_mods)
            )?;
            write!(
                f,
                "\n\tnegative_mod_mask = {};",
                keycodes::bitmod_to_name(self.negative_mod_mask)
            )?;
            write!(
                f,
                "\n\tsuppressed_mods = {};",
                keycodes::bitmod_to_name(self.suppressed_mods)
            )?;
            write!(
                f,
                "\n\tko_option_activation_trigger_down = {}",
                self.ko_option_activation_trigger_down
            )?;
            write!(
                f,
                "\n\tko_option_activation_required_mod_down = {}",
                self.ko_option_activation_required_mod_down
            )?;
            write!(
                f,
                "\n\tko_option_activation_negative_mod_up = {}",
                self.ko_option_activation_negative_mod_up
            )?;
            write!(f, "\n\tko_option_one_mod = {}", self.ko_option_one_mod)?;
            write!(
                f,
                "\n\tko_option_no_reregister_trigger = {}",
                self.ko_option_no_reregister_trigger
            )?;
            write!(
                f,
                "\n\tko_option_no_unregister_on_other_key_down = {}",
                self.ko_option_no_unregister_on_other_key_down
            )?;
            Ok(write!(f, "\n\tko_enabled = {}", self.ko_enabled)?)
        }
    }
}

pub fn load_key_overrides_from_json(
    key_overrides_json: &Value,
    vial_version: u32,
) -> Result<Vec<KeyOverride>, Box<dyn std::error::Error>> {
    let key_overrides = key_overrides_json
        .as_array()
        .ok_or("key_override should be an array")?;
    let mut result = Vec::new();
    for (i, key_override) in key_overrides.iter().enumerate() {
        result.push(KeyOverride::from_json(i as u8, key_override, vial_version)?);
    }
    Ok(result)
}

pub fn load_key_overrides(
    device: &HidDevice,
    count: u8,
) -> Result<Vec<KeyOverride>, Box<dyn std::error::Error>> {
    let mut keyoverrides: Vec<KeyOverride> = vec![];
    for idx in 0..count {
        match send_recv(
            device,
            &[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_DYNAMIC_ENTRY_OP,
                DYNAMIC_VIAL_KEY_OVERRIDE_GET,
                idx,
            ],
        ) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    let keyoverride = KeyOverride {
                        index: idx,
                        trigger: ((buff[2] as u16) << 8) + buff[1] as u16,
                        replacement: ((buff[4] as u16) << 8) + buff[3] as u16,
                        layers: ((buff[6] as u16) << 8) + buff[5] as u16,
                        trigger_mods: buff[7],
                        negative_mod_mask: buff[8],
                        suppressed_mods: buff[9],
                        ko_option_activation_trigger_down: buff[10] & (1 << 0) == (1 << 0),
                        ko_option_activation_required_mod_down: buff[10] & (1 << 1) == (1 << 1),
                        ko_option_activation_negative_mod_up: buff[10] & (1 << 2) == (1 << 2),
                        ko_option_one_mod: buff[10] & (1 << 3) == (1 << 3),
                        ko_option_no_reregister_trigger: buff[10] & (1 << 4) == (1 << 4),
                        ko_option_no_unregister_on_other_key_down: buff[10] & (1 << 5) == (1 << 5),
                        ko_enabled: (buff[10] & (1 << 7)) == (1 << 7),
                    };
                    keyoverrides.push(keyoverride)
                } else {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(keyoverrides)
}

pub fn set_key_override(
    device: &HidDevice,
    keyoverride: &KeyOverride,
) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_KEY_OVERRIDE_SET,
            keyoverride.index,
            (keyoverride.trigger & 0xFF) as u8,
            ((keyoverride.trigger >> 8) & 0xFF) as u8,
            (keyoverride.replacement & 0xFF) as u8,
            ((keyoverride.replacement >> 8) & 0xFF) as u8,
            (keyoverride.layers & 0xFF) as u8,
            ((keyoverride.layers >> 8) & 0xFF) as u8,
            keyoverride.trigger_mods,
            keyoverride.negative_mod_mask,
            keyoverride.suppressed_mods,
            keyoverride.options(),
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::HidError(e).into()),
    }
}
pub fn key_overrides_to_json(
    key_overrides: &Vec<KeyOverride>,
    vial_version: u32,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for key_override in key_overrides {
        result.push(json!({
            "trigger": keycodes::qid_to_name(key_override.trigger, vial_version),
            "replacement": keycodes::qid_to_name(key_override.replacement, vial_version),
            "layers": key_override.layers,
            "trigger_mods": key_override.trigger_mods,
            "negative_mod_mask": key_override.negative_mod_mask,
            "suppressed_mods": key_override.suppressed_mods,
            "options": key_override.options(),
        }))
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_positive() {
        let keyoverride = KeyOverride::from_string(
            9,
            &"trigger=KC_1; replacement=KC_2; layers=1; options=ko_enabled|ko_option_no_reregister_trigger;".to_string(),
            6,
        )
        .unwrap();
        assert_eq!(keyoverride.index, 9);
        assert_eq!(keycodes::qid_to_name(keyoverride.trigger, 6), "KC_1");
        assert_eq!(keycodes::qid_to_name(keyoverride.replacement, 6), "KC_2");
        assert_eq!(keyoverride.layers, 2);
        assert_eq!(keyoverride.ko_enabled, true);
        assert_eq!(keyoverride.ko_option_no_reregister_trigger, true);
    }

    #[test]
    fn test_from_string_full() {
        let s = "t=KC_A; r=KC_B; l=1|3; tm=LCTL; nmm=RCTL; sm=LALT; o=enabled|one_mod".to_string();
        let ko = KeyOverride::from_string(0, &s, 6).unwrap();
        assert_eq!(keycodes::qid_to_name(ko.trigger, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(ko.replacement, 6), "KC_B");
        assert_eq!(ko.layers, (1 << 1) | (1 << 3));
        assert_eq!(ko.trigger_mods, 1); // MOD_LCTL
        assert_eq!(ko.negative_mod_mask, 16); // MOD_RCTL
        assert_eq!(ko.suppressed_mods, 4); // MOD_LALT
        assert!(ko.ko_enabled);
        assert!(ko.ko_option_one_mod);
    }

    #[test]
    fn test_from_string_errors() {
        assert!(
            KeyOverride::from_string(0, &"t=KC_A; r".to_string(), 6).is_err(),
            "Missing ="
        );
        assert!(
            KeyOverride::from_string(0, &"foo=bar".to_string(), 6).is_err(),
            "Unknown key"
        );
        assert!(
            KeyOverride::from_string(0, &"t=INVALID".to_string(), 6).is_err(),
            "Invalid keycode"
        );
        assert!(
            KeyOverride::from_string(0, &"l=abc".to_string(), 6).is_err(),
            "Invalid layer"
        );
        assert!(
            KeyOverride::from_string(0, &"o=invalid_option".to_string(), 6).is_err(),
            "Unknown option"
        );
    }

    #[test]
    fn test_from_json_valid() {
        let json = json!({
            "trigger": "KC_A",
            "replacement": "KC_B",
            "layers": 5, // 1 | 4
            "trigger_mods": 1,
            "negative_mod_mask": 2,
            "suppressed_mods": 4,
            "options": 129 // enabled | trigger_down
        });
        let ko = KeyOverride::from_json(0, &json, 6).unwrap();
        assert_eq!(keycodes::qid_to_name(ko.trigger, 6), "KC_A");
        assert_eq!(keycodes::qid_to_name(ko.replacement, 6), "KC_B");
        assert_eq!(ko.layers, 5);
        assert_eq!(ko.trigger_mods, 1);
        assert_eq!(ko.negative_mod_mask, 2);
        assert_eq!(ko.suppressed_mods, 4);
        assert!(ko.ko_enabled);
        assert!(ko.ko_option_activation_trigger_down);
    }

    #[test]
    fn test_from_json_errors() {
        assert!(KeyOverride::from_json(0, &json!("not an object"), 6).is_err());
        assert!(KeyOverride::from_json(0, &json!({"trigger": 123}), 6).is_err());
        assert!(KeyOverride::from_json(0, &json!({"layers": "abc"}), 6).is_err());
        assert!(KeyOverride::from_json(0, &json!({"unknown_key": "KC_A"}), 6).is_err());
    }

    #[test]
    fn test_options_bitmask() {
        let mut ko = KeyOverride::empty(0);
        assert_eq!(ko.options(), 0);
        ko.ko_enabled = true;
        assert_eq!(ko.options(), 128); // 1 << 7
        ko.ko_option_one_mod = true;
        assert_eq!(ko.options(), 136); // 128 | 8
        ko.ko_option_no_unregister_on_other_key_down = true;
        assert_eq!(ko.options(), 168); // 136 | 32
    }

    #[test]
    fn test_empty_and_is_empty() {
        let empty_ko = KeyOverride::empty(0);
        assert!(empty_ko.is_empty());

        let mut non_empty = KeyOverride::empty(1);
        non_empty.trigger = keycodes::name_to_qid(&"KC_A".to_string(), 6).unwrap();
        assert!(!non_empty.is_empty());

        let mut non_empty2 = KeyOverride::empty(2);
        non_empty2.ko_enabled = true;
        assert!(!non_empty2.is_empty());
    }

    #[test]
    fn test_display() {
        let empty_ko = KeyOverride::empty(0);
        assert_eq!(format!("{}", empty_ko), "0) EMPTY");

        let ko = KeyOverride::from_string(1, &"t=KC_A;r=KC_B;l=2|4;tm=LCTL;o=enabled".to_string(), 6)
            .unwrap();
        let display_str = format!("{}", ko);
        assert!(display_str.contains("trigger = KC_A;"));
        assert!(display_str.contains("replacement = KC_B;"));
        assert!(display_str.contains("layers = 2|4;"));
        assert!(display_str.contains("\n\ttrigger_mods = MOD_BIT_LCTRL;"));
        assert!(display_str.contains("\n\tko_enabled = true"));
    }

    #[test]
    fn test_json_round_trip() {
        let mut ko1 = KeyOverride::empty(0);
        ko1.trigger = keycodes::name_to_qid(&"KC_A".to_string(), 6).unwrap();
        ko1.replacement = keycodes::name_to_qid(&"KC_B".to_string(), 6).unwrap();
        ko1.layers = 1;
        ko1.ko_enabled = true;

        let key_overrides = vec![ko1];
        let json_val = key_overrides_to_json(&key_overrides, 6).unwrap();
        let loaded_kos = load_key_overrides_from_json(&Value::Array(json_val), 6).unwrap();

        assert_eq!(key_overrides.len(), loaded_kos.len());
        assert_eq!(key_overrides[0].trigger, loaded_kos[0].trigger);
        assert_eq!(key_overrides[0].replacement, loaded_kos[0].replacement);
        assert_eq!(key_overrides[0].layers, loaded_kos[0].layers);
        assert_eq!(key_overrides[0].options(), loaded_kos[0].options());
    }
}
