use crate::keycodes;
use crate::protocol::{
    send, send_recv, ProtocolError, CMD_VIAL_DYNAMIC_ENTRY_OP, CMD_VIA_VIAL_PREFIX,
    DYNAMIC_VIAL_KEY_OVERRIDE_GET, DYNAMIC_VIAL_KEY_OVERRIDE_SET, VIA_UNHANDLED,
};
use hidapi::HidDevice;
use serde_json::{json, Value};
use std::fmt;

#[derive(Debug)]
pub struct KeyOverride {
    index: u8,
    trigger: u16,
    replacement: u16,
    layers: u16,
    trigger_mods: u8,
    negative_mod_mask: u8,
    suppressed_mods: u8,
    ko_option_activation_trigger_down: bool,
    ko_option_activation_required_mod_down: bool,
    ko_option_activation_negative_mod_up: bool,
    ko_option_one_mod: bool,
    ko_option_no_reregister_trigger: bool,
    ko_option_no_unregister_on_other_key_down: bool,
    ko_enabled: bool,
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

    pub fn from_strings(
        index: u8,
        keys: Vec<&str>,
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

        if keys.len() > 0 {
            for part in keys {
                let (left, right) = part.split_once("=").ok_or("each part should contain =")?;
                match left {
                    "trigger" | "t" => {
                        trigger = keycodes::name_to_qid(&right.to_string())?;
                    }
                    "replacement" | "r" => replacement = keycodes::name_to_qid(&right.to_string())?,
                    "layers" | "l" => {
                        for l in right.split("|") {
                            let layer: u8 = l.parse()?;
                            layers |= 1 << layer;
                        }
                    }
                    "trigger_mods" | "tm" | "m" => {
                        trigger_mods = keycodes::name_to_bitmod(&right.to_string())?
                    }
                    "negative_mod_mask" | "nmm" | "n" => {
                        negative_mod_mask = keycodes::name_to_bitmod(&right.to_string())?
                    }
                    "suppressed_mods" | "sm" | "s" => {
                        suppressed_mods = keycodes::name_to_bitmod(&right.to_string())?
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
                                _ => todo!(),
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
            index: index,
            trigger: trigger,
            replacement: replacement,
            layers: layers,
            trigger_mods: trigger_mods,
            negative_mod_mask: negative_mod_mask,
            suppressed_mods: suppressed_mods,
            ko_option_activation_trigger_down: ko_option_activation_trigger_down,
            ko_option_activation_required_mod_down: ko_option_activation_required_mod_down,
            ko_option_activation_negative_mod_up: ko_option_activation_negative_mod_up,
            ko_option_one_mod: ko_option_one_mod,
            ko_option_no_reregister_trigger: ko_option_no_reregister_trigger,
            ko_option_no_unregister_on_other_key_down: ko_option_no_unregister_on_other_key_down,
            ko_enabled: ko_enabled,
        })
    }

    pub fn from_json(
        index: u8,
        key_override_json: &Value,
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
                        &value
                            .as_str()
                            .ok_or("trigger value should be string")?
                            .to_string(),
                    )?;
                }
                "replacement" => {
                    replacement = keycodes::name_to_qid(
                        &value
                            .as_str()
                            .ok_or("replacement value should be string")?
                            .to_string(),
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
            index: index,
            trigger: trigger,
            replacement: replacement,
            layers: layers,
            trigger_mods: trigger_mods,
            negative_mod_mask: negative_mod_mask,
            suppressed_mods: suppressed_mods,
            ko_option_activation_trigger_down: ko_option_activation_trigger_down,
            ko_option_activation_required_mod_down: ko_option_activation_required_mod_down,
            ko_option_activation_negative_mod_up: ko_option_activation_negative_mod_up,
            ko_option_one_mod: ko_option_one_mod,
            ko_option_no_reregister_trigger: ko_option_no_reregister_trigger,
            ko_option_no_unregister_on_other_key_down: ko_option_no_unregister_on_other_key_down,
            ko_enabled: ko_enabled,
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
            write!(f, "trigger = {}; ", keycodes::qid_to_name(self.trigger))?;
            write!(
                f,
                "replacement = {}; ",
                keycodes::qid_to_name(self.replacement)
            )?;
            write!(f, "layers = ")?;
            let mut lne = false;
            for l in 0..16 {
                if self.layers & (1 << l) != 0 {
                    if lne == true {
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
) -> Result<Vec<KeyOverride>, Box<dyn std::error::Error>> {
    let key_overrides = key_overrides_json
        .as_array()
        .ok_or("key_override should be an array")?;
    let mut result = Vec::new();
    for (i, key_override) in key_overrides.iter().enumerate() {
        result.push(KeyOverride::from_json(i as u8, &key_override)?);
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
            &device,
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
            Err(e) => return Err(e.into()),
        }
    }
    Ok(keyoverrides)
}

pub fn set_key_override(
    device: &HidDevice,
    keyoverride: &KeyOverride,
) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        &device,
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
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for key_override in key_overrides {
        result.push(json!({
            "trigger": keycodes::qid_to_name(key_override.trigger),
            "replacement": keycodes::qid_to_name(key_override.replacement),
            "layers": key_override.layers,
            "trigger_mods": key_override.trigger_mods,
            "negative_mod_mask": key_override.negative_mod_mask,
            "suppressed_mods": key_override.suppressed_mods,
            "options": key_override.options(),
        }))
    }
    Ok(result)
}
