use crate::keycodes;
use crate::protocol::{
    CMD_VIA_VIAL_PREFIX, CMD_VIAL_DYNAMIC_ENTRY_OP, DYNAMIC_VIAL_TAP_DANCE_GET,
    DYNAMIC_VIAL_TAP_DANCE_SET, ProtocolError, VIA_UNHANDLED, send, send_recv,
};
use hidapi::HidDevice;
use serde_json::{Value, json};
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
    pub fn from_string(index: u8, value: &String) -> Result<TapDance, Box<dyn std::error::Error>> {
        let (keys_string, output) = value
            .split_once("~")
            .ok_or("tapping term in ms should be passed after ~")?;
        let tapping_term: u16 = output.replace(" ", "").parse()?;
        let keys: Vec<_> = keys_string.split("+").collect();

        let mut ks: [u16; 4] = [0x0; 4];
        for (idx, kn) in keys.iter().enumerate() {
            ks[idx] = keycodes::name_to_qid(&kn.to_string())?;
        }
        Ok(TapDance {
            index,
            tap: ks[0],
            hold: ks[1],
            double_tap: ks[2],
            tap_hold: ks[3],
            tapping_term,
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
            index,
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
            index,
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

pub fn tap_dances_to_json(
    tap_dances: &Vec<TapDance>,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for tap_dance in tap_dances {
        result.push(json!([
            keycodes::qid_to_name(tap_dance.tap),
            keycodes::qid_to_name(tap_dance.hold),
            keycodes::qid_to_name(tap_dance.double_tap),
            keycodes::qid_to_name(tap_dance.tap_hold),
            tap_dance.tapping_term,
        ]))
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tap_hold() {
        let tapdance = TapDance::from_string(7, &"KC_V + KC_B ~ 50".to_string()).unwrap();
        assert_eq!(tapdance.index, 7);
        assert_eq!(keycodes::qid_to_name(tapdance.tap), "KC_V");
        assert_eq!(keycodes::qid_to_name(tapdance.hold), "KC_B");
        assert_eq!(tapdance.double_tap, 0);
        assert_eq!(tapdance.tap_hold, 0);
        assert_eq!(tapdance.tapping_term, 50);
    }

    #[test]
    fn test_from_string_one_key() {
        let td = TapDance::from_string(0, &"KC_A ~ 100".to_string()).unwrap();
        assert_eq!(keycodes::qid_to_name(td.tap), "KC_A");
        assert_eq!(td.hold, 0);
        assert_eq!(td.tapping_term, 100);
    }

    #[test]
    fn test_from_string_four_keys() {
        let td = TapDance::from_string(1, &"KC_A+KC_B+KC_C+KC_D ~ 200".to_string()).unwrap();
        assert_eq!(keycodes::qid_to_name(td.tap), "KC_A");
        assert_eq!(keycodes::qid_to_name(td.hold), "KC_B");
        assert_eq!(keycodes::qid_to_name(td.double_tap), "KC_C");
        assert_eq!(keycodes::qid_to_name(td.tap_hold), "KC_D");
        assert_eq!(td.tapping_term, 200);
    }

    #[test]
    fn test_from_string_errors() {
        assert!(
            TapDance::from_string(0, &"KC_A".to_string()).is_err(),
            "Missing ~"
        );
        assert!(
            TapDance::from_string(0, &"KC_A ~ abc".to_string()).is_err(),
            "Invalid tapping term"
        );
        assert!(
            TapDance::from_string(0, &"INVALID ~ 100".to_string()).is_err(),
            "Invalid keycode"
        );
    }

    #[test]
    fn test_from_json_valid() {
        let json = json!(["KC_A", "KC_B", "KC_C", "KC_D", 250]);
        let td = TapDance::from_json(0, &json).unwrap();
        assert_eq!(keycodes::qid_to_name(td.tap), "KC_A");
        assert_eq!(keycodes::qid_to_name(td.hold), "KC_B");
        assert_eq!(keycodes::qid_to_name(td.double_tap), "KC_C");
        assert_eq!(keycodes::qid_to_name(td.tap_hold), "KC_D");
        assert_eq!(td.tapping_term, 250);
    }

    #[test]
    fn test_from_json_errors() {
        assert!(
            TapDance::from_json(0, &json!("KC_A")).is_err(),
            "Not an array"
        );

        // A short array is not an error, it just fills with KC_NO
        let td = TapDance::from_json(0, &json!(["KC_A"])).unwrap();
        assert_eq!(keycodes::qid_to_name(td.tap), "KC_A");
        assert_eq!(td.hold, 0);

        assert!(
            TapDance::from_json(0, &json!(["KC_A", "KC_B", "KC_C", "KC_D", 200, "KC_E"])).is_err(),
            "Array too long"
        );
        assert!(
            TapDance::from_json(0, &json!(["KC_A", "KC_B", "KC_C", "KC_D", "KC_E"])).is_err(),
            "Tapping term not a number"
        );
        assert!(
            TapDance::from_json(0, &json!([1, 2, 3, 4, 100])).is_err(),
            "Keycode not a string"
        );
        assert!(
            TapDance::from_json(0, &json!(["INVALID", "KC_B", "KC_C", "KC_D", 100])).is_err(),
            "Invalid keycode"
        );
    }

    #[test]
    fn test_empty_and_is_empty() {
        let empty_td = TapDance::empty(0);
        assert!(empty_td.is_empty());
        assert_eq!(empty_td.tapping_term, 0);

        let non_empty_td = TapDance::from_string(1, &"KC_A ~ 100".to_string()).unwrap();
        assert!(!non_empty_td.is_empty());
    }

    #[test]
    fn test_display() {
        let empty_td = TapDance::empty(0);
        assert_eq!(format!("{}", empty_td), "0) EMPTY");

        let full_td =
            TapDance::from_string(1, &"KC_A + KC_B + KC_C + KC_D ~ 200".to_string()).unwrap();
        assert_eq!(
            format!("{}", full_td),
            "1) On tap: KC_A, On hold: KC_B, On double tap: KC_C, On tap + hold: KC_D, Tapping term (ms) = 200"
        );

        let partial_td = TapDance::from_string(2, &"KC_A + KC_NO ~ 150".to_string()).unwrap();
        assert_eq!(
            format!("{}", partial_td),
            "2) On tap: KC_A, Tapping term (ms) = 150"
        );
    }

    #[test]
    fn test_json_round_trip() {
        let td1 = TapDance::from_string(0, &"KC_A + KC_B ~ 100".to_string()).unwrap();
        let td2 = TapDance::from_string(1, &"KC_C + KC_D + KC_E ~ 200".to_string()).unwrap();
        let tap_dances = vec![td1, td2];

        let json_val = tap_dances_to_json(&tap_dances).unwrap();
        let loaded_tap_dances = load_tap_dances_from_json(&Value::Array(json_val)).unwrap();

        assert_eq!(tap_dances.len(), loaded_tap_dances.len());
        assert_eq!(
            format!("{}", tap_dances[0]),
            format!("{}", loaded_tap_dances[0])
        );
        assert_eq!(
            format!("{}", tap_dances[1]),
            format!("{}", loaded_tap_dances[1])
        );
    }
}
