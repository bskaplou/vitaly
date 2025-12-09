use thiserror::Error;

mod v5;
mod v6;

#[allow(dead_code)]
#[derive(Error, Debug)]
#[error("{0}")]
pub struct KeyParsingError(pub String);

const MOD_BIT_LCTRL: u8 = 0b00000001;
const MOD_BIT_LSHIFT: u8 = 0b00000010;
const MOD_BIT_LALT: u8 = 0b00000100;
const MOD_BIT_LGUI: u8 = 0b00001000;
const MOD_BIT_RCTRL: u8 = 0b00010000;
const MOD_BIT_RSHIFT: u8 = 0b00100000;
const MOD_BIT_RALT: u8 = 0b01000000;
const MOD_BIT_RGUI: u8 = 0b10000000;

pub fn name_to_bitmod(mods: &str) -> Result<u8, KeyParsingError> {
    let mut m = 0x0u8;
    for mp in mods.split("|") {
        match mp {
            "MOD_BIT_LCTRL" | "MOD_LCTL" | "LCTL" | "LC" | "CTL" | "C" => m |= MOD_BIT_LCTRL,
            "MOD_BIT_LSHIFT" | "MOD_LSFT" | "LSFT" | "LS" | "SFT" | "S" => m |= MOD_BIT_LSHIFT,
            "MOD_BIT_LALT" | "MOD_LALT" | "LALT" | "LA" | "ALT" | "A" => m |= MOD_BIT_LALT,
            "MOD_BIT_LGUI" | "MOD_LGUI" | "LGUI" | "LG" | "GUI" | "G" => m |= MOD_BIT_LGUI,
            "MOD_BIT_RCTRL" | "MOD_RCTL" | "RCTL" | "RC" => m |= MOD_BIT_RCTRL,
            "MOD_BIT_RSHIFT" | "MOD_RSFT" | "RSFT" | "RS" => m |= MOD_BIT_RSHIFT,
            "MOD_BIT_RALT" | "MOD_RALT" | "RALT" | "RA" => m |= MOD_BIT_RALT,
            "MOD_BIT_RGUI" | "MOD_RGUI" | "RGUI" | "RG" => m |= MOD_BIT_RGUI,
            &_ => {
                return Err(KeyParsingError(
                    format!("can't parse mod {}", mp).to_string(),
                ));
            }
        }
    }
    Ok(m)
}

pub fn bitmod_to_name(modcode: u8) -> String {
    let mut dest = String::new();
    if modcode & MOD_BIT_RCTRL == MOD_BIT_RCTRL {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RCTRL");
    }
    if modcode & MOD_BIT_LCTRL == MOD_BIT_LCTRL {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LCTRL");
    }
    if modcode & MOD_BIT_RSHIFT == MOD_BIT_RSHIFT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RSHIFT");
    }
    if modcode & MOD_BIT_LSHIFT == MOD_BIT_LSHIFT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LSHIFT");
    }
    if modcode & MOD_BIT_RALT == MOD_BIT_RALT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RALT");
    }
    if modcode & MOD_BIT_LALT == MOD_BIT_LALT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LALT");
    }
    if modcode & MOD_BIT_RGUI == MOD_BIT_RGUI {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RGUI");
    }
    if modcode & MOD_BIT_LGUI == MOD_BIT_LGUI {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LGUI");
    }
    if dest.is_empty() {
        dest.push_str("KC_NO");
    }
    dest
}

const MOD_LCTL: u8 = 0x01;
const MOD_LSFT: u8 = 0x02;
const MOD_LALT: u8 = 0x04;
const MOD_LGUI: u8 = 0x08;
const MOD_RCTL: u8 = 0x11;
const MOD_RSFT: u8 = 0x12;
const MOD_RALT: u8 = 0x14;
const MOD_RGUI: u8 = 0x18;

fn name_to_mod(mods: &str) -> Result<u8, KeyParsingError> {
    let mut m = 0x0u8;
    for mp in mods.split("|") {
        match mp {
            "MOD_LCTL" | "LCTL" | "CTL" | "C" => m |= MOD_LCTL,
            "MOD_LSFT" | "LSFT" | "SFT" | "S" => m |= MOD_LSFT,
            "MOD_LALT" | "LALT" | "ALT" | "A" => m |= MOD_LALT,
            "MOD_LGUI" | "LGUI" | "GUI" | "G" => m |= MOD_LGUI,
            "MOD_RCTL" | "RCTL" => m |= MOD_RCTL,
            "MOD_RSFT" | "RSFT" => m |= MOD_RSFT,
            "MOD_RALT" | "RALT" => m |= MOD_RALT,
            "MOD_RGUI" | "RGUI" => m |= MOD_RGUI,
            &_ => {
                return Err(KeyParsingError(
                    format!("can't parse mod {}", mp).to_string(),
                ));
            }
        }
    }
    Ok(m)
}

pub fn mod_to_name(modcode: u8) -> String {
    let mut dest = String::new();
    if modcode & MOD_RCTL == MOD_RCTL {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_RCTL");
    } else if modcode & MOD_LCTL == MOD_LCTL {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_LCTL");
    }
    if modcode & MOD_RSFT == MOD_RSFT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_RSFT");
    } else if modcode & MOD_LSFT == MOD_LSFT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_LSFT");
    }
    if modcode & MOD_RALT == MOD_RALT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_RALT");
    } else if modcode & MOD_LALT == MOD_LALT {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_LALT");
    }
    if modcode & MOD_RGUI == MOD_RGUI {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_RGUI");
    } else if modcode & MOD_LGUI == MOD_LGUI {
        if !dest.is_empty() {
            dest.push('|');
        }
        dest.push_str("MOD_LGUI");
    }
    if dest.is_empty() {
        dest.push_str("KC_NO");
    }
    dest
}

fn parse_layer(layer: &String) -> Result<u16, KeyParsingError> {
    let parsed: Result<u16, _> = layer.parse();
    match parsed {
        Ok(p) => Ok(p),
        Err(_) => Err(KeyParsingError(
            format!("can't parse layer {} should be num", layer).to_string(),
        )),
    }
}

fn parse_num(num: &String) -> Result<u16, KeyParsingError> {
    let parsed: Result<u16, _> = num.parse();
    match parsed {
        Ok(p) => Ok(p),
        Err(_) => Err(KeyParsingError(
            format!("can't parse argument {} should be num", num).to_string(),
        )),
    }
}

pub fn name_to_qid(name: &str, vial_version: u32) -> Result<u16, Box<dyn std::error::Error>> {
    match vial_version {
        6 => v6::name_to_qid(name),
        _ => v5::name_to_qid(name),
    }
}

pub fn qid_to_short(keycode: u16, vial_version: u32) -> String {
    match vial_version {
        6 => v6::qid_to_short(keycode),
        _ => v5::qid_to_short(keycode),
    }
}

pub fn qid_to_name(keycode: u16, vial_version: u32) -> String {
    match vial_version {
        6 => v6::qid_to_name(keycode),
        _ => v5::qid_to_name(keycode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_5_to_name() {
        assert_eq!(qid_to_name(0x7228, 5), "MT(MOD_RSFT,KC_ENTER)");
        assert_eq!(qid_to_name(0x5f10, 5), "QK_TRI_LAYER_LOWER");
        assert_eq!(qid_to_name(0x5f11, 5), "QK_TRI_LAYER_UPPER");
        assert_eq!(qid_to_name(0x5f1c, 5), "QK_MACRO_10");
        assert_eq!(qid_to_name(0x5c14, 5), "QK_MAGIC_TOGGLE_NKRO");
    }

    #[test]
    fn test_5_to_short() {
        assert_eq!(qid_to_short(0x7228, 5), "MT(MOD_RSFT,KC_ENTER)");
        assert_eq!(qid_to_short(0x5f10, 5), "Fn1,Fn3");
        assert_eq!(qid_to_short(0x5f11, 5), "Fn2,Fn3");
        assert_eq!(qid_to_short(0x5f1c, 5), "QK_MACRO_10");
    }

    #[test]
    fn test_5_to_code() {
        assert_eq!(name_to_qid("MT(MOD_RSFT,KC_ENTER)", 5).unwrap(), 0x7228);
        assert_eq!(name_to_qid("QK_TRI_LAYER_LOWER", 5).unwrap(), 0x5f10);
        assert_eq!(name_to_qid("QK_TRI_LAYER_UPPER", 5).unwrap(), 0x5f11);
        assert_eq!(name_to_qid("QK_MACRO_10", 5).unwrap(), 0x5f1c);
        assert_eq!(name_to_qid("QK_MAGIC_TOGGLE_NKRO", 5).unwrap(), 0x5c14);
    }

    #[test]
    fn test_6_to_name() {
        assert_eq!(qid_to_name(0x3228, 6), "MT(MOD_RSFT,KC_ENTER)");
        assert_eq!(qid_to_name(0x7C77, 6), "QK_TRI_LAYER_LOWER");
        assert_eq!(qid_to_name(0x7C78, 6), "QK_TRI_LAYER_UPPER");
        assert_eq!(qid_to_name(0x770A, 6), "QK_MACRO_10");
        assert_eq!(qid_to_name(0x7013, 6), "QK_MAGIC_TOGGLE_NKRO");
    }

    #[test]
    fn test_6_to_short() {
        assert_eq!(qid_to_short(0x3228, 6), "MT(MOD_RSFT,KC_ENTER)");
        assert_eq!(qid_to_short(0x7C77, 6), "Fn1,Fn3");
        assert_eq!(qid_to_short(0x7C78, 6), "Fn2,Fn3");
        assert_eq!(qid_to_short(0x770A, 6), "QK_MACRO_10");
    }

    #[test]
    fn test_6_to_code() {
        assert_eq!(name_to_qid("MT(MOD_RSFT,KC_ENTER)", 6).unwrap(), 0x3228);
        assert_eq!(name_to_qid("QK_TRI_LAYER_LOWER", 6).unwrap(), 0x7C77);
        assert_eq!(name_to_qid("QK_TRI_LAYER_UPPER", 6).unwrap(), 0x7C78);
        assert_eq!(name_to_qid("QK_MACRO_10", 6).unwrap(), 0x770A);
        assert_eq!(name_to_qid("QK_MAGIC_TOGGLE_NKRO", 6).unwrap(), 0x7013);
    }
}
