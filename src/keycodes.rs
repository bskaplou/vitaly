use thiserror::Error;

pub mod code_to_name;
pub mod name_to_code;

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
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RCTRL");
    }
    if modcode & MOD_BIT_LCTRL == MOD_BIT_LCTRL {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LCTRL");
    }
    if modcode & MOD_BIT_RSHIFT == MOD_BIT_RSHIFT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RSHIFT");
    }
    if modcode & MOD_BIT_LSHIFT == MOD_BIT_LSHIFT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LSHIFT");
    }
    if modcode & MOD_BIT_RALT == MOD_BIT_RALT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RALT");
    }
    if modcode & MOD_BIT_LALT == MOD_BIT_LALT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LALT");
    }
    if modcode & MOD_BIT_RGUI == MOD_BIT_RGUI {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_RGUI");
    }
    if modcode & MOD_BIT_LGUI == MOD_BIT_LGUI {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_BIT_LGUI");
    }
    if dest.len() == 0 {
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
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_RCTL");
    } else if modcode & MOD_LCTL == MOD_LCTL {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_LCTL");
    }
    if modcode & MOD_RSFT == MOD_RSFT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_RSFT");
    } else if modcode & MOD_LSFT == MOD_LSFT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_LSFT");
    }
    if modcode & MOD_RALT == MOD_RALT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_RALT");
    } else if modcode & MOD_LALT == MOD_LALT {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_LALT");
    }
    if modcode & MOD_RGUI == MOD_RGUI {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_RGUI");
    } else if modcode & MOD_LGUI == MOD_LGUI {
        if dest.len() > 0 {
            dest.push('|');
        }
        dest.push_str("MOD_LGUI");
    }
    if dest.len() == 0 {
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
            format!("can't argument {} should be num", num).to_string(),
        )),
    }
}

pub fn name_to_qid(name: &String) -> Result<u16, Box<dyn std::error::Error>> {
    let n = name.replace(" ", "");
    if let Some((left, right_str)) = n.split_once('(') {
        let keycode;
        let mut right_s = right_str.to_string();
        right_s.pop(); // kill closing )
        let right = right_s.to_owned();
        match left {
            "QK_LCTL" | "LCTL" | "C" => {
                keycode = 0x0100u16 | name_to_qid(&right.to_string())?;
            }
            "QK_LSFT" | "LSFT" | "S" => {
                keycode = 0x0200u16 | name_to_qid(&right.to_string())?;
            }
            "QK_LALT" | "LALT" | "LOPT" | "A" => {
                keycode = 0x0400u16 | name_to_qid(&right.to_string())?;
            }
            "QK_LGUI" | "LGUI" | "LCMD" | "LWIN" | "G" => {
                keycode = 0x0800u16 | name_to_qid(&right.to_string())?;
            }
            "QK_RCTL" | "RCTL" => {
                keycode = 0x1100u16 | name_to_qid(&right.to_string())?;
            }
            "QK_RSFT" | "RSFT" => {
                keycode = 0x1200u16 | name_to_qid(&right.to_string())?;
            }
            "QK_RALT" | "RALT" | "ALGR" | "ROPT" => {
                keycode = 0x1400u16 | name_to_qid(&right.to_string())?;
            }
            "QK_RGUI" | "RGUI" | "RCMD" | "RWIN" => {
                keycode = 0x1800u16 | name_to_qid(&right.to_string())?;
            }
            "HYPR" => {
                keycode = 0x0F00u16 | name_to_qid(&right.to_string())?;
            }
            "MEH" => {
                keycode = 0x0700u16 | name_to_qid(&right.to_string())?;
            }
            "LCAG" => {
                keycode = 0x0D00u16 | name_to_qid(&right.to_string())?;
            }
            "LSG" | "SGUI" | "SCMD" | "SWIN" => {
                keycode = 0x0A00u16 | name_to_qid(&right.to_string())?;
            }
            "LAG" => {
                keycode = 0x0C00u16 | name_to_qid(&right.to_string())?;
            }
            "RSG" => {
                keycode = 0x1A00u16 | name_to_qid(&right.to_string())?;
            }
            "RAG" => {
                keycode = 0x1C00u16 | name_to_qid(&right.to_string())?;
            }
            "LCA" => {
                keycode = 0x0500u16 | name_to_qid(&right.to_string())?;
            }
            "LSA" => {
                keycode = 0x0600u16 | name_to_qid(&right.to_string())?;
            }
            "RSA" | "SAGR" => {
                keycode = 0x1600u16 | name_to_qid(&right.to_string())?;
            }
            "RCS" => {
                keycode = 0x1300u16 | name_to_qid(&right.to_string())?;
            }
            "TO" => {
                keycode = 0x5200 | (parse_layer(&right)? & 0x1F);
            }
            "MO" => {
                keycode = 0x5220 | (parse_layer(&right)? & 0x1F);
            }
            "DF" => {
                keycode = 0x5240 | (parse_layer(&right)? & 0x1F);
            }
            "PDF" => {
                keycode = 0x52E0 | (parse_layer(&right)? & 0x1F);
            }
            "TG" => {
                keycode = 0x5260 | (parse_layer(&right)? & 0x1F);
            }
            "OSL" => {
                keycode = 0x5260 | (parse_layer(&right)? & 0x1F);
            }
            "LM" => match right.split_once(",") {
                None => {
                    return Err(KeyParsingError(
                        format!(
                            "LM should have strictly two arguments {:?} doesn't match",
                            right
                        )
                        .to_string(),
                    )
                    .into());
                }
                Some((layer, mo)) => {
                    let l: u16 = parse_layer(&layer.to_string())?;
                    let m = name_to_mod(mo)? as u16;
                    keycode = 0x5000 | ((l & 0xF) << 5) | (m & 0x1F);
                }
            },
            "OSM" => {
                let m = name_to_mod(&right)? as u16;
                keycode = 0x52A0 | (m & 0x1F);
            }
            "TT" => {
                keycode = 0x52C0 | (parse_layer(&right)? & 0x1F);
            }
            "LT" => match right.split_once(",") {
                None => {
                    return Err(KeyParsingError(
                        format!(
                            "LT should have strictly two arguments {:?} doesn't match",
                            right
                        )
                        .to_string(),
                    )
                    .into());
                }
                Some((layer, key)) => {
                    let l: u16 = parse_layer(&layer.to_string())?;
                    let k = name_to_qid(&key.to_string())?;
                    keycode = 0x4000 | ((l & 0x0F) << 8) | (k & 0xFF);
                }
            },
            "MT" => match right.split_once(",") {
                None => {
                    return Err(KeyParsingError(
                        format!(
                            "MT should have strictly two arguments {:?} doesn't match",
                            right
                        )
                        .to_string(),
                    )
                    .into());
                }
                Some((mods, key)) => {
                    let m = name_to_mod(&mods.to_string())? as u16;
                    let k = name_to_qid(&key.to_string())?;
                    keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
                }
            },
            "LCTL_T" | "CTL_T" => {
                let m = MOD_LCTL as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RCTL_T" => {
                let m = MOD_RCTL as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LSFT_T" | "SFT_T" => {
                let m = MOD_LSFT as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RSFT_T" => {
                let m = MOD_RSFT as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LALT_T" | "ALT_T" | "LOPT_T" | "OPT_T" => {
                let m = MOD_LALT as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RALT_T" | "ROPT_T" | "ALGR_T" => {
                let m = MOD_RALT as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LGUI_T" | "GUI_T" | "LCMD_T" | "CMD_T" | "LWIN_T" | "WIN_T" => {
                let m = MOD_LGUI as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RGUI_T" | "RCMD_T" | "RWIN_T" => {
                let m = MOD_RGUI as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "C_S_T" => {
                let m = (MOD_LCTL | MOD_LSFT) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "MEH_T" => {
                let m = (MOD_LCTL | MOD_LSFT | MOD_LALT) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LCAG_T" => {
                let m = (MOD_LCTL | MOD_LALT | MOD_LGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RCAG_T" => {
                let m = (MOD_RCTL | MOD_RALT | MOD_RGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "HYPR_T" | "ALL_T" => {
                let m = (MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LSG_T" | "SGUI_T" | "SCMD_T" | "SWIN_T" => {
                let m = (MOD_LSFT | MOD_LGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LAG_T" => {
                let m = (MOD_LALT | MOD_LGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RSG_T" => {
                let m = (MOD_RSFT | MOD_RGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RAG_T" => {
                let m = (MOD_RALT | MOD_RGUI) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LCA_T" => {
                let m = (MOD_LCTL | MOD_LALT) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "LSA_T" => {
                let m = (MOD_LSFT | MOD_LALT) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RSA_T" | "SAGR_T" => {
                let m = (MOD_RSFT | MOD_RALT) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "RCS_T" => {
                let m = (MOD_RCTL | MOD_RSFT) as u16;
                let k = name_to_qid(&right.to_string())?;
                keycode = 0x2000 | ((m & 0x1F) << 8) | (k & 0xFF);
            }
            "TD" => {
                let i: u16 = parse_num(&right.to_string())?;
                keycode = 0x5700 | (i & 0xFF);
            }
            &_ => {
                return Err(
                    KeyParsingError(format!("can't find macro {}", left).to_string()).into(),
                );
            }
        }
        Ok(keycode)
    } else {
        match name_to_code::FULLNAMES.get(n.as_str()) {
            Some(value) => Ok(*value),
            None => Err(KeyParsingError(format!("can't find key {}", n).to_string()).into()),
        }
    }
}

pub fn qid_to_short(keycode: u16) -> String {
    let mut dest = String::new();
    match keycode {
        0x0200..=0x02FF => {
            dest.push_str("L⇧,");
            dest.push_str(&qid_to_short(keycode & 0xFF));
        }
        0x1200..=0x12FF => {
            dest.push_str("R⇧,");
            dest.push_str(&qid_to_short(keycode & 0xFF));
        }
        0x5200..=0x521F => {
            dest.push_str("TO,");
            dest.push_str((keycode & 0x1F).to_string().as_str());
        }
        0x5220..=0x523F => {
            dest.push_str("MO,");
            dest.push_str((keycode & 0x1F).to_string().as_str());
        }
        0x5260..=0x527F => {
            dest.push_str("TG,");
            dest.push_str((keycode & 0x1F).to_string().as_str());
        }
        _ => match code_to_name::SHORTNAMES.get(&keycode) {
            Some(name) => {
                dest.push_str(*name);
            }
            None => return qid_to_name(keycode),
        },
    }
    dest
}

pub fn qid_to_name(keycode: u16) -> String {
    let mut dest = String::new();
    match keycode {
        0x0100..=0x01FF => {
            dest.push_str("LCTL(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x0200..=0x02FF => {
            dest.push_str("LSFT(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x0400..=0x04FF => {
            dest.push_str("LALT(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x0800..=0x08FF => {
            dest.push_str("LGUI(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x1100..=0x11FF => {
            dest.push_str("RCTL(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x1200..=0x12FF => {
            dest.push_str("RSFT(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x1400..=0x14FF => {
            dest.push_str("RALT(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x1800..=0x18FF => {
            dest.push_str("RGUI(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        //HYPR 0x0f00
        0x0F00..=0x0FFF => {
            dest.push_str("HYPR(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        //MEH 0x0700
        0x0700..=0x07FF => {
            dest.push_str("MEH(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // LCAG 0x0d00
        0x0D00..=0x0DFF => {
            dest.push_str("LCAG(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // LSG 0x0a00
        0x0A00..=0x0AFF => {
            dest.push_str("LSG(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // LAG 0x0c00
        0x0C00..=0x0CFF => {
            dest.push_str("LAG(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // RSG 0x1a00
        0x1A00..=0x1AFF => {
            dest.push_str("RSG(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // RAG 0x1c00
        0x1C00..=0x1CFF => {
            dest.push_str("RAG(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // LCA 0x0500
        0x0500..=0x05FF => {
            dest.push_str("LCA(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // LSA 0x0600
        0x0600..=0x06FF => {
            dest.push_str("LSA(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // RSA 0x1600
        0x1600..=0x16FF => {
            dest.push_str("RSA(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        // RCS 0x1300
        0x1300..=0x13FF => {
            dest.push_str("RCS(");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x5200..=0x521F => {
            dest.push_str("TO(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x5220..=0x523F => {
            dest.push_str("MO(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x5240..=0x525F => {
            dest.push_str("DF(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x52E0..=0x52FF => {
            dest.push_str("PDF(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x5260..=0x527F => {
            dest.push_str("TG(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x5280..=0x529F => {
            dest.push_str("OSL(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x5000..=0x51FF => {
            dest.push_str("LM(");
            dest.push_str(((keycode >> 5) & 0xF).to_string().as_str());
            dest.push_str(",");
            dest.push_str(mod_to_name((keycode & 0x1F) as u8).as_str());
            dest.push_str(")");
        }
        0x52A0..=0x52BF => {
            dest.push_str("OSM(");
            dest.push_str(mod_to_name((keycode & 0x1F) as u8).as_str());
            dest.push_str(")");
        }
        0x52C0..=0x52DF => {
            dest.push_str("TT(");
            dest.push_str((keycode & 0x1F).to_string().as_str());
            dest.push_str(")");
        }
        0x4000..=0x4FFF => {
            dest.push_str("LT(");
            dest.push_str(((keycode >> 8) & 0x0F).to_string().as_str());
            dest.push_str(",");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x2000..=0x3FFF => {
            dest.push_str("MT(");
            dest.push_str(mod_to_name(((keycode >> 8) & 0x1F) as u8).as_str());
            dest.push_str(",");
            dest.push_str(&qid_to_name(keycode & 0xFF));
            dest.push_str(")");
        }
        0x5700..=0x57FF => {
            dest.push_str("TD(");
            dest.push_str((keycode & 0xFF).to_string().as_str());
            dest.push_str(")");
        }
        _ => match code_to_name::FULLNAMES.get(&keycode) {
            Some(name) => {
                dest.push_str(*name);
            }
            None => {
                println!("fixme {:#04x}", keycode);
                dest.push_str("UNKNOWN");
            }
        },
    }
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_bitmod_lctrl() {
        assert_eq!(name_to_bitmod("MOD_BIT_LCTRL").unwrap(), MOD_BIT_LCTRL);
        assert_eq!(name_to_bitmod("MOD_LCTL").unwrap(), MOD_BIT_LCTRL);
        assert_eq!(name_to_bitmod("LCTL").unwrap(), MOD_BIT_LCTRL);
        assert_eq!(name_to_bitmod("LC").unwrap(), MOD_BIT_LCTRL);
        assert_eq!(name_to_bitmod("CTL").unwrap(), MOD_BIT_LCTRL);
        assert_eq!(name_to_bitmod("C").unwrap(), MOD_BIT_LCTRL);
    }

    #[test]
    fn test_name_to_bitmod_lshift() {
        assert_eq!(name_to_bitmod("MOD_BIT_LSHIFT").unwrap(), MOD_BIT_LSHIFT);
        assert_eq!(name_to_bitmod("MOD_LSFT").unwrap(), MOD_BIT_LSHIFT);
        assert_eq!(name_to_bitmod("LSFT").unwrap(), MOD_BIT_LSHIFT);
        assert_eq!(name_to_bitmod("LS").unwrap(), MOD_BIT_LSHIFT);
        assert_eq!(name_to_bitmod("SFT").unwrap(), MOD_BIT_LSHIFT);
        assert_eq!(name_to_bitmod("S").unwrap(), MOD_BIT_LSHIFT);
    }

    #[test]
    fn test_name_to_bitmod_lalt() {
        assert_eq!(name_to_bitmod("MOD_BIT_LALT").unwrap(), MOD_BIT_LALT);
        assert_eq!(name_to_bitmod("MOD_LALT").unwrap(), MOD_BIT_LALT);
        assert_eq!(name_to_bitmod("LALT").unwrap(), MOD_BIT_LALT);
        assert_eq!(name_to_bitmod("LA").unwrap(), MOD_BIT_LALT);
        assert_eq!(name_to_bitmod("ALT").unwrap(), MOD_BIT_LALT);
        assert_eq!(name_to_bitmod("A").unwrap(), MOD_BIT_LALT);
    }

    #[test]
    fn test_name_to_bitmod_lgui() {
        assert_eq!(name_to_bitmod("MOD_BIT_LGUI").unwrap(), MOD_BIT_LGUI);
        assert_eq!(name_to_bitmod("MOD_LGUI").unwrap(), MOD_BIT_LGUI);
        assert_eq!(name_to_bitmod("LGUI").unwrap(), MOD_BIT_LGUI);
        assert_eq!(name_to_bitmod("LG").unwrap(), MOD_BIT_LGUI);
        assert_eq!(name_to_bitmod("GUI").unwrap(), MOD_BIT_LGUI);
        assert_eq!(name_to_bitmod("G").unwrap(), MOD_BIT_LGUI);
    }

    #[test]
    fn test_name_to_bitmod_rctrl() {
        assert_eq!(name_to_bitmod("MOD_BIT_RCTRL").unwrap(), MOD_BIT_RCTRL);
        assert_eq!(name_to_bitmod("MOD_RCTL").unwrap(), MOD_BIT_RCTRL);
        assert_eq!(name_to_bitmod("RCTL").unwrap(), MOD_BIT_RCTRL);
        assert_eq!(name_to_bitmod("RC").unwrap(), MOD_BIT_RCTRL);
    }

    #[test]
    fn test_name_to_bitmod_rshift() {
        assert_eq!(name_to_bitmod("MOD_BIT_RSHIFT").unwrap(), MOD_BIT_RSHIFT);
        assert_eq!(name_to_bitmod("MOD_RSFT").unwrap(), MOD_BIT_RSHIFT);
        assert_eq!(name_to_bitmod("RSFT").unwrap(), MOD_BIT_RSHIFT);
        assert_eq!(name_to_bitmod("RS").unwrap(), MOD_BIT_RSHIFT);
    }

    #[test]
    fn test_name_to_bitmod_ralt() {
        assert_eq!(name_to_bitmod("MOD_BIT_RALT").unwrap(), MOD_BIT_RALT);
        assert_eq!(name_to_bitmod("MOD_RALT").unwrap(), MOD_BIT_RALT);
        assert_eq!(name_to_bitmod("RALT").unwrap(), MOD_BIT_RALT);
        assert_eq!(name_to_bitmod("RA").unwrap(), MOD_BIT_RALT);
    }

    #[test]
    fn test_name_to_bitmod_rgui() {
        assert_eq!(name_to_bitmod("MOD_BIT_RGUI").unwrap(), MOD_BIT_RGUI);
        assert_eq!(name_to_bitmod("MOD_RGUI").unwrap(), MOD_BIT_RGUI);
        assert_eq!(name_to_bitmod("RGUI").unwrap(), MOD_BIT_RGUI);
        assert_eq!(name_to_bitmod("RG").unwrap(), MOD_BIT_RGUI);
    }

    #[test]
    fn test_name_to_bitmod_or() {
        assert_eq!(
            name_to_bitmod("C|S").unwrap(),
            MOD_BIT_LCTRL | MOD_BIT_LSHIFT
        );
        assert_eq!(name_to_bitmod("C|G").unwrap(), MOD_BIT_LCTRL | MOD_BIT_LGUI);
    }

    #[test]
    fn test_name_to_bitmod_neg1() -> Result<(), String> {
        match name_to_bitmod("unknown") {
            Err(_) => Ok(()),
            Ok(_) => Err("should be error".to_string()),
        }
    }

    #[test]
    fn test_name_to_bitmod_neg2() -> Result<(), String> {
        match name_to_bitmod("MOD_BIT_LSHIFT|unknown") {
            Err(_) => Ok(()),
            Ok(_) => Err("should be error".to_string()),
        }
    }

    #[test]
    fn test_bitmod_to_name_raw() {
        assert_eq!("MOD_BIT_LCTRL", bitmod_to_name(MOD_BIT_LCTRL));
        assert_eq!("MOD_BIT_LSHIFT", bitmod_to_name(MOD_BIT_LSHIFT));
        assert_eq!("MOD_BIT_LALT", bitmod_to_name(MOD_BIT_LALT));
        assert_eq!("MOD_BIT_LGUI", bitmod_to_name(MOD_BIT_LGUI));
        assert_eq!("MOD_BIT_RCTRL", bitmod_to_name(MOD_BIT_RCTRL));
        assert_eq!("MOD_BIT_RSHIFT", bitmod_to_name(MOD_BIT_RSHIFT));
        assert_eq!("MOD_BIT_RALT", bitmod_to_name(MOD_BIT_RALT));
        assert_eq!("MOD_BIT_RGUI", bitmod_to_name(MOD_BIT_RGUI));
    }

    #[test]
    fn test_bitmod_to_name_or() {
        assert_eq!(
            "MOD_BIT_LCTRL|MOD_BIT_LSHIFT",
            bitmod_to_name(MOD_BIT_LCTRL | MOD_BIT_LSHIFT)
        );
    }

    #[test]
    fn test_bitmod_to_name_no() {
        assert_eq!("KC_NO", bitmod_to_name(0));
    }

    #[test]
    fn test_name_to_mod_raw() {
        assert_eq!(name_to_mod("MOD_LCTL").unwrap(), MOD_LCTL);
        assert_eq!(name_to_mod("MOD_LSFT").unwrap(), MOD_LSFT);
        assert_eq!(name_to_mod("MOD_LALT").unwrap(), MOD_LALT);
        assert_eq!(name_to_mod("MOD_LGUI").unwrap(), MOD_LGUI);

        assert_eq!(name_to_mod("MOD_RCTL").unwrap(), MOD_RCTL);
        assert_eq!(name_to_mod("MOD_RSFT").unwrap(), MOD_RSFT);
        assert_eq!(name_to_mod("MOD_RALT").unwrap(), MOD_RALT);
        assert_eq!(name_to_mod("MOD_RGUI").unwrap(), MOD_RGUI);
    }

    #[test]
    fn test_name_to_mod_or() {
        assert_eq!(
            name_to_mod("MOD_LSFT|MOD_LCTL").unwrap(),
            MOD_LCTL | MOD_LSFT
        );
        assert_eq!(
            name_to_mod("MOD_LGUI|MOD_LCTL").unwrap(),
            MOD_LCTL | MOD_LGUI
        );
        assert_eq!(
            name_to_mod("MOD_RGUI|MOD_LCTL").unwrap(),
            MOD_RCTL | MOD_LGUI
        );
        assert_eq!(
            name_to_mod("MOD_RCTL|MOD_RALT").unwrap(),
            MOD_RCTL | MOD_RALT
        );
    }

    #[test]
    fn test_name_to_mod_neg1() -> Result<(), String> {
        match name_to_mod("unknown") {
            Err(_) => Ok(()),
            Ok(_) => Err("should be error".to_string()),
        }
    }

    #[test]
    fn test_name_to_mod_neg2() -> Result<(), String> {
        match name_to_mod("MOD_LSFT|unknown") {
            Err(_) => Ok(()),
            Ok(_) => Err("should be error".to_string()),
        }
    }

    #[test]
    fn test_mod_to_name_raw() {
        assert_eq!("MOD_LCTL", mod_to_name(MOD_LCTL));
        assert_eq!("MOD_LSFT", mod_to_name(MOD_LSFT));
        assert_eq!("MOD_LALT", mod_to_name(MOD_LALT));
        assert_eq!("MOD_LGUI", mod_to_name(MOD_LGUI));
        assert_eq!("MOD_RCTL", mod_to_name(MOD_RCTL));
        assert_eq!("MOD_RSFT", mod_to_name(MOD_RSFT));
        assert_eq!("MOD_RALT", mod_to_name(MOD_RALT));
        assert_eq!("MOD_RGUI", mod_to_name(MOD_RGUI));
    }

    #[test]
    fn test_mod_to_name_or() {
        assert_eq!("MOD_LCTL|MOD_LSFT", mod_to_name(MOD_LCTL | MOD_LSFT));
        assert_eq!("MOD_RCTL|MOD_RSFT", mod_to_name(MOD_RCTL | MOD_RSFT));
        assert_eq!("MOD_RCTL|MOD_RSFT", mod_to_name(MOD_LCTL | MOD_RSFT));
        assert_eq!("MOD_RSFT|MOD_RALT", mod_to_name(MOD_RALT | MOD_RSFT));
        assert_eq!("MOD_RSFT|MOD_RGUI", mod_to_name(MOD_RGUI | MOD_RSFT));
    }

    #[test]
    fn test_mod_to_name_no() {
        assert_eq!("KC_NO", mod_to_name(0));
    }

    #[test]
    fn test_qid_no() {
        assert_eq!("KC_NO", qid_to_name(0));
    }

    #[test]
    fn test_qid_neg() -> Result<(), String> {
        match name_to_qid(&"unknown".to_string()) {
            Err(_) => Ok(()),
            Ok(_) => Err("shoud be error".to_string()),
        }
    }
    #[test]
    fn test_qid() {
        for s in [
            "KC_1",
            "LCTL(KC_1)",
            "LSFT(KC_GRAVE)",
            "LALT(KC_COMMA)",
            "RALT(KC_COMMA)",
            "MT(MOD_LSFT,KC_2)",
            "TD(1)",
            "TG(2)",
            "MO(3)",
            "TO(4)",
            "RSA(KC_6)",
            "RCS(KC_I)",
            "HYPR(KC_NO)",
            "MEH(KC_D)",
            "LAG(KC_F)",
            "MT(MOD_LCTL|MOD_LSFT,KC_C)",
            "LT(1,KC_D)",
        ] {
            assert_eq!(qid_to_name(name_to_qid(&s.to_string()).unwrap()), s);
        }
    }
}
