use crate::keycodes;
use crate::protocol::{
    send, send_recv, ProtocolError, CMD_VIAL_DYNAMIC_ENTRY_OP, CMD_VIA_VIAL_PREFIX,
    DYNAMIC_VIAL_TAP_DANCE_GET, DYNAMIC_VIAL_TAP_DANCE_SET, VIA_UNHANDLED,
};
use hidapi::HidDevice;
use serde_json::Value;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct TapDanceFormatError(pub String);

#[derive(Debug)]
pub struct TapDance {
    pub index: u8,
    pub tap: u16,
    pub hold: u16,
    pub double_tap: u16,
    pub tap_hold: u16,
    pub tapping_term: u16,
}

impl TapDance {
    pub fn from_strings(
        index: u8,
        keys: Vec<&str>,
        tapping_term: u16,
    ) -> Result<TapDance, Box<dyn std::error::Error>> {
        let mut ks: [u16; 4] = [0x0; 4];
        for (idx, kn) in keys.iter().enumerate() {
            ks[idx] = keycodes::name_to_qid(&kn.to_string())?;
        }
        Ok(TapDance {
            index: index,
            tap: ks[0],
            hold: ks[1],
            double_tap: ks[2],
            tap_hold: ks[3],
            tapping_term: tapping_term,
        })
    }

    pub fn from_json(
        index: u8,
        tap_dances_json: &Value,
    ) -> Result<TapDance, Box<dyn std::error::Error>> {
        let mut ks: [u16; 5] = [0x0; 5];
        let values = tap_dances_json
            .as_array()
            .ok_or("TapDances should be encoded into array")?;
        for (pos, val) in values.iter().enumerate() {
            match pos {
                0 | 1 | 2 | 3 => {
                    let value_string = val
                        .as_str()
                        .ok_or("TapDance elements 0-3 should be strings")?;
                    let qid = keycodes::name_to_qid(&value_string.to_string())?;
                    ks[pos] = qid
                }
                4 => {
                    ks[pos] = val
                        .as_u64()
                        .ok_or("TapDance 3th element should be positive number")?
                        as u16
                }
                _ => {
                    return Err(TapDanceFormatError(
                        "TapDance array should be strictly 5 elements long".to_string(),
                    )
                    .into());
                }
            }
        }
        Ok(TapDance {
            index: index,
            tap: ks[0],
            hold: ks[1],
            double_tap: ks[2],
            tap_hold: ks[3],
            tapping_term: ks[4],
        })
    }

    pub fn is_empty(&self) -> bool {
        self.tap == 0 && self.hold == 0 && self.double_tap == 0 && self.tap_hold == 0
    }

    pub fn empty(index: u8) -> TapDance {
        TapDance {
            index: index,
            tap: 0,
            hold: 0,
            double_tap: 0,
            tap_hold: 0,
            tapping_term: 0,
        }
    }
}

impl fmt::Display for TapDance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.is_empty() {
            Ok(write!(f, "EMPTY")?)
        } else {
            if self.tap != 0 {
                write!(f, "On tap: {}, ", keycodes::qid_to_name(self.tap))?
            }
            if self.hold != 0 {
                write!(f, "On hold: {}, ", keycodes::qid_to_name(self.hold))?
            }
            if self.double_tap != 0 {
                write!(
                    f,
                    "On double tap: {}, ",
                    keycodes::qid_to_name(self.double_tap)
                )?
            }
            if self.tap_hold != 0 {
                write!(
                    f,
                    "On tap + hold: {}, ",
                    keycodes::qid_to_name(self.tap_hold)
                )?
            }
            Ok(write!(f, "Tapping term (ms) = {}", self.tapping_term)?)
        }
    }
}

pub fn load_tap_dances(
    device: &HidDevice,
    count: u8,
) -> Result<Vec<TapDance>, Box<dyn std::error::Error>> {
    let mut tapdances: Vec<TapDance> = vec![];
    for idx in 0..count {
        match send_recv(
            &device,
            &[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_DYNAMIC_ENTRY_OP,
                DYNAMIC_VIAL_TAP_DANCE_GET,
                idx,
            ],
        ) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    let tapdance = TapDance {
                        index: idx,
                        tap: ((buff[2] as u16) << 8) + buff[1] as u16,
                        hold: ((buff[4] as u16) << 8) + buff[3] as u16,
                        double_tap: ((buff[6] as u16) << 8) + buff[5] as u16,
                        tap_hold: ((buff[8] as u16) << 8) + buff[7] as u16,
                        tapping_term: ((buff[10] as u16) << 8) + buff[9] as u16,
                    };
                    tapdances.push(tapdance)
                } else {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(tapdances)
}

pub fn load_tap_dances_from_json(
    tap_dances_json: &Value,
) -> Result<Vec<TapDance>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let tap_dances = tap_dances_json
        .as_array()
        .ok_or("TapDances should be encoded as array")?;
    for (i, tap_dance) in tap_dances.iter().enumerate() {
        result.push(TapDance::from_json(i as u8, tap_dance)?)
    }
    Ok(result)
}

pub fn set_tap_dance(
    device: &HidDevice,
    tapdance: &TapDance,
) -> Result<(), Box<dyn std::error::Error>> {
    match send(
        &device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_TAP_DANCE_SET,
            tapdance.index,
            (tapdance.tap & 0xFF) as u8,
            ((tapdance.tap >> 8) & 0xFF) as u8,
            (tapdance.hold & 0xFF) as u8,
            ((tapdance.hold >> 8) & 0xFF) as u8,
            (tapdance.double_tap & 0xFF) as u8,
            ((tapdance.double_tap >> 8) & 0xFF) as u8,
            (tapdance.tap_hold & 0xFF) as u8,
            ((tapdance.tap_hold >> 8) & 0xFF) as u8,
            (tapdance.tapping_term & 0xFF) as u8,
            ((tapdance.tapping_term >> 8) & 0xFF) as u8,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::HidError(e).into()),
    }
}
