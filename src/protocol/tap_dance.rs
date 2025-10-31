use crate::protocol::keycodes;
use crate::protocol::{
    send, send_recv, ProtocolError, CMD_VIAL_DYNAMIC_ENTRY_OP, CMD_VIA_VIAL_PREFIX,
    DYNAMIC_VIAL_TAP_DANCE_GET, DYNAMIC_VIAL_TAP_DANCE_SET, VIA_UNHANDLED,
};
use hidapi::HidDevice;
use std::fmt;

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
    ) -> Result<TapDance, keycodes::KeyParsingError> {
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

    pub fn empty(&self) -> bool {
        self.tap == 0 && self.hold == 0 && self.double_tap == 0 && self.tap_hold == 0
    }
}

impl fmt::Display for TapDance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.empty() {
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
