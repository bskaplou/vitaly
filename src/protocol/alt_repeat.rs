use crate::keycodes;
use crate::protocol::{
    send, send_recv, ProtocolError, CMD_VIAL_DYNAMIC_ENTRY_OP, CMD_VIA_VIAL_PREFIX,
    DYNAMIC_VIAL_ALT_REPEAT_KEY_GET, DYNAMIC_VIAL_ALT_REPEAT_KEY_SET, VIA_UNHANDLED,
};
use hidapi::HidDevice;
use serde_json::{json, Value};
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

    pub fn from_strings(
        index: u8,
        keys: Vec<&str>,
    ) -> Result<AltRepeat, Box<dyn std::error::Error>> {
        let mut keycode = 0u16;
        let mut alt_keycode = 0u16;
        let mut allowed_mods = 0u8;
        let mut arep_option_default_to_this_alt_key = false;
        let mut arep_option_bidirectional = false;
        let mut arep_option_ignore_mod_handedness = false;
        let mut arep_enabled = false;

        if keys.len() > 0 {
            for part in keys {
                let (left, right) = part.split_once("=").ok_or("each part should contain =")?;
                match left {
                    "keycode" | "k" => keycode = keycodes::name_to_qid(&right.to_string())?,
                    "alt_keycode" | "a" => alt_keycode = keycodes::name_to_qid(&right.to_string())?,
                    "allowed_mods" | "m" => {
                        allowed_mods = keycodes::name_to_bitmod(&right.to_string())?
                    }
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
            index: index,
            keycode: keycode,
            alt_keycode: alt_keycode,
            allowed_mods: allowed_mods,
            arep_option_default_to_this_alt_key: arep_option_default_to_this_alt_key,
            arep_option_bidirectional: arep_option_bidirectional,
            arep_option_ignore_mod_handedness: arep_option_ignore_mod_handedness,
            arep_enabled: arep_enabled,
        })
    }

    pub fn from_json(
        index: u8,
        alt_repeat_json: &Value,
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
                        &value
                            .as_str()
                            .ok_or("keycode value should be string")?
                            .to_string(),
                    )?;
                }
                "alt_keycode" => {
                    alt_keycode = keycodes::name_to_qid(
                        &value
                            .as_str()
                            .ok_or("keycode value should be string")?
                            .to_string(),
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
            index: index,
            keycode: keycode,
            alt_keycode: alt_keycode,
            allowed_mods: allowed_mods,
            arep_option_default_to_this_alt_key: arep_option_default_to_this_alt_key,
            arep_option_bidirectional: arep_option_bidirectional,
            arep_option_ignore_mod_handedness: arep_option_ignore_mod_handedness,
            arep_enabled: arep_enabled,
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
            write!(f, "keycode = {}; ", keycodes::qid_to_name(self.keycode))?;
            write!(
                f,
                "alt_keycode = {}; ",
                keycodes::qid_to_name(self.alt_keycode)
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
            &device,
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
            Err(e) => return Err(e.into()),
        }
    }
    Ok(altrepeats)
}

pub fn load_alt_repeats_from_json(
    alt_repeats_json: &Value,
) -> Result<Vec<AltRepeat>, Box<dyn std::error::Error>> {
    let alt_repeats = alt_repeats_json
        .as_array()
        .ok_or("alt_repeats_json should be an array")?;
    let mut result = Vec::new();
    for (i, alt_repeat) in alt_repeats.iter().enumerate() {
        result.push(AltRepeat::from_json(i as u8, &alt_repeat)?);
    }
    Ok(result)
}

pub fn set_alt_repeat(
    device: &HidDevice,
    altrepeat: &AltRepeat,
) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        &device,
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
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for alt_repeat in alt_repeats {
        result.push(json!({
            "keycode": keycodes::qid_to_name(alt_repeat.keycode),
            "alt_keycode": keycodes::qid_to_name(alt_repeat.alt_keycode),
            "allowed_mods": alt_repeat.allowed_mods,
            "options": alt_repeat.options(),
        }))
    }
    Ok(result)
}
