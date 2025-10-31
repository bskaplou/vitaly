use crate::protocol::keycodes;
use crate::protocol::{
    send, send_recv, ProtocolError, CMD_VIAL_DYNAMIC_ENTRY_OP, CMD_VIA_VIAL_PREFIX,
    DYNAMIC_VIAL_ALT_REPEAT_KEY_GET, DYNAMIC_VIAL_ALT_REPEAT_KEY_SET, VIA_UNHANDLED,
};
use hidapi::HidDevice;
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
    pub fn from_strings(
        index: u8,
        keys: Vec<&str>,
    ) -> Result<AltRepeat, keycodes::KeyParsingError> {
        let mut keycode = 0u16;
        let mut alt_keycode = 0u16;
        let mut allowed_mods = 0u8;
        let mut arep_option_default_to_this_alt_key = false;
        let mut arep_option_bidirectional = false;
        let mut arep_option_ignore_mod_handedness = false;
        let mut arep_enabled = false;

        if keys.len() > 0 {
            for part in keys {
                let (left, right) = part.split_once("=").unwrap();
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
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(keycodes::KeyParsingError(
                            format!("Unknown setting {}", left).to_string(),
                        ));
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

    pub fn empty(&self) -> bool {
        self.keycode == 0 && !self.arep_enabled
    }
}

impl fmt::Display for AltRepeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.empty() {
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

pub fn set_alt_repeat(
    device: &HidDevice,
    altrepeat: &AltRepeat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = 0u8;
    if altrepeat.arep_option_default_to_this_alt_key {
        options |= 1;
    }
    if altrepeat.arep_option_bidirectional {
        options |= 1 << 1;
    }
    if altrepeat.arep_option_ignore_mod_handedness {
        options |= 1 << 2;
    }
    if altrepeat.arep_enabled {
        options |= 1 << 3;
    }
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
            options,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::HidError(e).into()),
    }
}
