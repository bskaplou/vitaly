use crate::keycodes;
use crate::protocol::{
    BUFFER_FETCH_CHUNK, CMD_VIA_MACRO_GET_BUFFER, CMD_VIA_MACRO_SET_BUFFER, Capabilities,
    MESSAGE_LENGTH, ProtocolError, VIA_UNHANDLED, send_recv,
};
use hidapi::HidDevice;
use serde_json::{Value, json};
use std::cmp::min;
use std::fmt;
use thiserror::Error;

const SS_QMK_PREFIX: u8 = 1;
const SS_TAP_CODE: u8 = 1;
const SS_DOWN_CODE: u8 = 2;
const SS_UP_CODE: u8 = 3;
const SS_DELAY_CODE: u8 = 4;
const VIAL_MACRO_EXT_TAP: u8 = 5;
const VIAL_MACRO_EXT_DOWN: u8 = 6;
const VIAL_MACRO_EXT_UP: u8 = 7;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct MacroParsingError(String);

#[derive(Error, Debug)]
#[error("{0}")]
pub struct MacroSavingError(String);

#[derive(Debug)]
pub enum MacroStep {
    Tap(u16),
    Down(u16),
    Up(u16),
    Delay(u16),
    Text(String),
}

impl MacroStep {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            MacroStep::Delay(ms) => {
                result.push(SS_QMK_PREFIX);
                result.push(SS_DELAY_CODE);
                let d1 = ms % 255 + 1;
                let d2 = ms / 255 + 1;
                result.push(d1 as u8);
                result.push(d2 as u8);
            }
            MacroStep::Text(txt) => {
                result.extend(txt.as_bytes());
            }
            MacroStep::Tap(kc) | MacroStep::Down(kc) | MacroStep::Up(kc) => {
                result.push(SS_QMK_PREFIX);
                if *kc < 256 {
                    let cmd = match self {
                        MacroStep::Tap(_) => SS_TAP_CODE,
                        MacroStep::Down(_) => SS_DOWN_CODE,
                        MacroStep::Up(_) => SS_UP_CODE,
                        _ => 42,
                    };
                    result.push(cmd);
                    result.push(*kc as u8)
                } else {
                    let cmd = match self {
                        MacroStep::Tap(_) => VIAL_MACRO_EXT_TAP,
                        MacroStep::Down(_) => VIAL_MACRO_EXT_DOWN,
                        MacroStep::Up(_) => VIAL_MACRO_EXT_UP,
                        _ => 42,
                    };
                    result.push(cmd);
                    let c = if kc % 256 == 0 {
                        0xFF00 | (kc >> 8)
                    } else {
                        *kc
                    };
                    result.push((c & 0xFF) as u8);
                    result.push(((c >> 8) & 0xFF) as u8);
                }
            }
        }
        result
    }

    fn from_string(step: &str, vial_version: u32) -> Result<MacroStep, Box<dyn std::error::Error>> {
        let (left, right) = step.split_once("(").ok_or("Lack of parenthesis")?;
        let right = right[0..(right.len() - 1)].to_string();
        match left {
            "Delay" => Ok(MacroStep::Delay(right.parse()?)),
            "Text" => Ok(MacroStep::Text(right)),
            "Tap" => Ok(MacroStep::Tap(keycodes::name_to_qid(&right, vial_version)?)),
            "Down" => Ok(MacroStep::Down(keycodes::name_to_qid(&right, vial_version)?)),
            "Up" => Ok(MacroStep::Up(keycodes::name_to_qid(&right, vial_version)?)),
            _ => Err(MacroParsingError(format!("Unknown macro step {}", right).to_string()).into()),
        }
    }

    fn from_json(step_json: &Value, vial_version: u32) -> Result<Vec<MacroStep>, Box<dyn std::error::Error>> {
        let step = step_json
            .as_array()
            .ok_or("macro step should be an array")?;
        if step.len() < 2 {
            return Err(MacroParsingError(
                "macro step array should be at least 2 elements long".to_string(),
            )
            .into());
        }
        let mut result = Vec::new();
        let action = &step[0];
        match action.as_str().ok_or("action should be string")? {
            "delay" => {
                for arg in &step[1..] {
                    result.push(MacroStep::Delay(
                        arg.as_u64().ok_or("delay argument should be number")? as u16,
                    ));
                }
                Ok(result)
            }
            "text" => {
                for arg in &step[1..] {
                    let text_arg = arg.as_str().ok_or("text argument should be string");
                    result.push(MacroStep::Text(text_arg?.to_string()));
                }
                Ok(result)
            }
            "tap" => {
                for arg in &step[1..] {
                    let text_arg = arg.as_str().ok_or("text argument should be string")?;
                    result.push(MacroStep::Tap(keycodes::name_to_qid(text_arg, vial_version)?));
                }
                Ok(result)
            }
            "down" => {
                for arg in &step[1..] {
                    let text_arg = arg.as_str().ok_or("text argument should be string")?;
                    result.push(MacroStep::Down(keycodes::name_to_qid(text_arg, vial_version)?));
                }
                Ok(result)
            }
            "up" => {
                for arg in &step[1..] {
                    let text_arg = arg.as_str().ok_or("text argument should be string")?;
                    result.push(MacroStep::Up(keycodes::name_to_qid(text_arg, vial_version)?));
                }
                Ok(result)
            }
            action => {
                Err(MacroParsingError(format!("Unknown macro step {}", action).to_string()).into())
            }
        }
    }
}

impl fmt::Display for MacroStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MacroStep::Tap(kc) => Ok(write!(f, "Tap({})", keycodes::qid_to_name(*kc, 6))?),
            MacroStep::Down(kc) => Ok(write!(f, "Down({})", keycodes::qid_to_name(*kc, 6))?),
            MacroStep::Up(kc) => Ok(write!(f, "Up({})", keycodes::qid_to_name(*kc, 6))?),
            MacroStep::Delay(ms) => Ok(write!(f, "Delay({})", ms)?),
            MacroStep::Text(txt) => Ok(write!(f, "Text({})", txt)?),
        }
    }
}

#[derive(Debug)]
pub struct Macro {
    pub index: u8,
    pub steps: Vec<MacroStep>,
}

impl Macro {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for step in &self.steps {
            result.append(&mut step.serialize());
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.steps.len() == 0
    }

    pub fn from_string(index: u8, value: &str, vial_version: u32) -> Result<Macro, Box<dyn std::error::Error>> {
        let steps: Vec<&str> = value.split(";").map(|s| s.trim()).collect();
        let mut parsed_steps = Vec::new();
        for step in steps {
            if !step.is_empty() {
                parsed_steps.push(MacroStep::from_string(step, vial_version)?)
            }
        }
        Ok(Macro {
            index,
            steps: parsed_steps,
        })
    }

    pub fn from_json(index: u8, steps_json: &Value, vial_version: u32) -> Result<Macro, Box<dyn std::error::Error>> {
        let mut parsed_steps = Vec::new();
        let steps = steps_json
            .as_array()
            .ok_or("macro should be defined as array of macro steps")?;
        for step in steps {
            parsed_steps.append(&mut MacroStep::from_json(step, vial_version)?);
        }
        Ok(Macro {
            index,
            steps: parsed_steps,
        })
    }
}

impl fmt::Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}) ", self.index)?;
        if self.is_empty() {
            Ok(write!(f, "EMPTY")?)
        } else {
            for (i, step) in self.steps.iter().enumerate() {
                if i > 0 {
                    write!(f, "; ")?
                }
                write!(f, "{}", step)?
            }
            Ok(())
        }
    }
}

pub fn load_macros_from_json(
    macros_json: &Value,
    vial_version: u32
) -> Result<Vec<Macro>, Box<dyn std::error::Error>> {
    let macros = macros_json
        .as_array()
        .ok_or("macro value should be an array")?;
    let mut result = Vec::new();
    for (i, m) in macros.iter().enumerate() {
        result.push(Macro::from_json(i as u8, m, vial_version)?);
    }
    Ok(result)
}

// State machine here
enum MacroParsingState {
    Start,
    Text(usize),
    NextCommand,
    Command(u8),
    CommandWithArgs(u8, u8),
}

fn deserialize_single(index: u8, data: &[u8]) -> Result<Macro, Box<dyn std::error::Error>> {
    let mut steps = Vec::new();
    let mut s: MacroParsingState = MacroParsingState::Start;
    for i in 0..data.len() {
        match s {
            MacroParsingState::Start => match data[i] {
                SS_QMK_PREFIX => s = MacroParsingState::NextCommand,
                _ => s = MacroParsingState::Text(i),
            },
            MacroParsingState::Text(start_index) => match data[i] {
                SS_QMK_PREFIX => {
                    let step = MacroStep::Text(str::from_utf8(&data[start_index..i])?.to_string());
                    steps.push(step);
                    s = MacroParsingState::NextCommand
                }
                _ => {
                    // text goes on
                }
            },
            MacroParsingState::NextCommand => s = MacroParsingState::Command(data[i]),
            MacroParsingState::Command(cmd) => {
                if cmd == SS_DELAY_CODE
                    || cmd == VIAL_MACRO_EXT_TAP
                    || cmd == VIAL_MACRO_EXT_DOWN
                    || cmd == VIAL_MACRO_EXT_UP
                {
                    s = MacroParsingState::CommandWithArgs(cmd, data[i])
                } else {
                    let step = match cmd {
                        SS_TAP_CODE => MacroStep::Tap(data[i] as u16),
                        SS_DOWN_CODE => MacroStep::Down(data[i] as u16),
                        SS_UP_CODE => MacroStep::Up(data[i] as u16),
                        _ => {
                            return Err(MacroParsingError(
                                format!("Unknown command {}", cmd).to_string(),
                            )
                            .into());
                        }
                    };
                    steps.push(step);
                    s = MacroParsingState::Start
                }
            }
            MacroParsingState::CommandWithArgs(cmd, arg1) => {
                let arg2 = data[i];
                let mut kc = (arg1 as u16) + ((arg2 as u16) << 8);
                if kc > 0xFF00 {
                    kc = (kc & 0xFF) << 8
                }
                let step = match cmd {
                    SS_DELAY_CODE => {
                        MacroStep::Delay(((arg2 as u16) - 1) * 255 + ((arg1 as u16) - 1))
                    }
                    VIAL_MACRO_EXT_TAP => MacroStep::Tap(kc),
                    VIAL_MACRO_EXT_DOWN => MacroStep::Down(kc),
                    VIAL_MACRO_EXT_UP => MacroStep::Up(kc),
                    _ => {
                        return Err(MacroParsingError(
                            format!("Unknown command {}", cmd).to_string(),
                        )
                        .into());
                    }
                };
                steps.push(step);
                s = MacroParsingState::Start
            }
        }
    }
    match s {
        MacroParsingState::Start => {
            // Fine! Last command wasn't text
        }
        MacroParsingState::Text(start_index) => {
            let step = MacroStep::Text(str::from_utf8(&data[start_index..data.len()])?.to_string());
            steps.push(step)
        }
        _ => return Err(MacroParsingError("Unexpected state after last byte".to_string()).into()),
    }
    Ok(Macro { index, steps })
}

pub fn deserialize(data: Vec<u8>) -> Result<Vec<Macro>, Box<dyn std::error::Error>> {
    let mut start = 0;
    let mut pos = 0;
    let mut macroses = Vec::new();
    if !(data.is_empty() || data.len() == 1 && data[0] == 0) {
        for i in 0..data.len() {
            if data[i] == 0 {
                let macro_bytes = data.get(start..i).ok_or("fatal deserialization error")?;
                let m = deserialize_single(pos, macro_bytes)?;
                macroses.push(m);
                pos += 1;
                start = i + 1;
            }
        }
    }
    Ok(macroses)
}

pub fn serialize(macros: &Vec<Macro>) -> Vec<u8> {
    let mut result = Vec::new();
    for m in macros {
        result.extend(m.serialize());
        result.push(0)
    }
    result
}

pub fn load_macros(
    device: &HidDevice,
    count: u8,
    buffer_size: u16,
) -> Result<Vec<Macro>, Box<dyn std::error::Error>> {
    let mut macro_buffer = Vec::new();
    let mut macro_loaded = 0;
    let mut last_zero = false;

    'load: loop {
        let loaded: u16 = macro_buffer.len() as u16;
        let l1: u8 = ((loaded >> 8) & 0xFF) as u8;
        let l2: u8 = (loaded & 0xFF) as u8;
        let read_size = min(buffer_size - loaded, BUFFER_FETCH_CHUNK as u16) as u8;
        if read_size == 0 {
            break 'load;
        }

        match send_recv(device, &[CMD_VIA_MACRO_GET_BUFFER, l1, l2, read_size]) {
            Ok(buff) => {
                if buff[0] != VIA_UNHANDLED {
                    for i in 4..(read_size + 4) {
                        if buff[i as usize] == 0 {
                            if last_zero {
                                macro_buffer.extend_from_slice(&buff[4..i as usize]);
                                break 'load;
                            } else {
                                last_zero = true;
                            }
                            macro_loaded += 1;
                            if macro_loaded == count {
                                macro_buffer.extend_from_slice(&buff[4..=i as usize]);
                                break 'load;
                            }
                        } else {
                            last_zero = false;
                        }
                    }
                    macro_buffer.extend_from_slice(&buff[4..(read_size + 4) as usize]);
                } else {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
            }
            Err(e) => return Err(e),
        }
    }
    deserialize(macro_buffer)
    /*
    let d = deserialize(macro_buffer.clone())?;
    let dd = deserialize(macro_buffer.clone())?;
    println!(">>>{:?}", macro_buffer);
    println!("<<<{:?}", serialize(d));
    Ok(dd)
    */
}

pub fn set_macros(
    device: &HidDevice,
    capabilities: &Capabilities,
    macros: &Vec<Macro>,
) -> Result<(), Box<dyn std::error::Error>> {
    if macros.len() > capabilities.macro_count.into() {
        return Err(MacroSavingError(
            format!(
                "Not enough macro buffer size: macro count = {}, allowed macro count = {}",
                macros.len(),
                capabilities.macro_count
            )
            .to_string(),
        )
        .into());
    }
    let data = serialize(macros);
    if data.len() > capabilities.macro_buffer_size.into() {
        return Err(MacroSavingError(
            format!(
                "Not enough macro buffer size: macros length = {}, allowed buffer size = {}",
                data.len(),
                capabilities.macro_buffer_size
            )
            .to_string(),
        )
        .into());
    }
    // println!(">>{:?}", data);
    let mut offset: u16 = 0;
    while offset < capabilities.macro_buffer_size {
        let mut msg: [u8; MESSAGE_LENGTH] = [0u8; MESSAGE_LENGTH];
        let to_send = min(
            capabilities.macro_buffer_size - offset,
            BUFFER_FETCH_CHUNK as u16,
        ) as u8;
        msg[0] = CMD_VIA_MACRO_SET_BUFFER;
        msg[1] = ((offset >> 8) & 0xFF) as u8;
        msg[2] = (offset & 0xFF) as u8;
        msg[3] = to_send;
        for i in 0..to_send {
            let data_shift = offset as usize + (i as usize);
            if data_shift < data.len() {
                msg[(i + 4) as usize] = data[offset as usize + (i as usize)];
            }
        }
        /*
        println!(
            "offset: {:?}, to_send: {:?}, data: {:?}",
            offset, to_send, msg
        );
        */
        match send_recv(device, &msg) {
            Ok(buff) => {
                if buff[0] == VIA_UNHANDLED {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
                // Fine!
            }
            Err(e) => return Err(e),
        }
        offset += to_send as u16;
    }
    Ok(())
}

pub fn macros_to_json(macros: &Vec<Macro>, vial_version: u32) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for m in macros {
        let mut step_json = Vec::new();
        for step in &m.steps {
            match step {
                MacroStep::Tap(v) => step_json.push(json!(["tap", keycodes::qid_to_name(*v, vial_version)])),
                MacroStep::Down(v) => step_json.push(json!(["down", keycodes::qid_to_name(*v, vial_version)])),
                MacroStep::Up(v) => step_json.push(json!(["up", keycodes::qid_to_name(*v, vial_version)])),
                MacroStep::Delay(v) => step_json.push(json!(["delay", v])),
                MacroStep::Text(v) => step_json.push(json!(["text", v])),
            }
        }
        result.push(serde_json::Value::Array(step_json));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keycodes::name_to_qid;

    #[test]
    fn test_serde1() {
        let b1: &[u8] = &[1, 2, 14, 1, 2, 206, 1, 3, 206, 116, 101, 115, 116, 0];
        assert_eq!(b1, serialize(&deserialize(b1.to_vec()).unwrap()));
    }

    #[test]
    fn test_serde2() {
        let b1: &[u8] = &[
            1, 4, 6, 1, 1, 4, 236, 4, 0, 116, 101, 115, 116, 0, 1, 2, 30, 0, 1, 3, 30, 0, 1, 1, 30,
            0, 84, 69, 83, 84, 1, 4, 101, 1, 0, 1, 5, 126, 255, 0,
        ];
        assert_eq!(b1, serialize(&deserialize(b1.to_vec()).unwrap()));
    }

    #[test]
    fn test_from_string() -> Result<(), Box<dyn std::error::Error>> {
        let m = Macro::from_string(12, &"Text(example); Tap(KC_1)".to_string(), 6)?;
        assert_eq!(12, m.index);
        assert_eq!(2, m.steps.len());
        Ok(())
    }

    fn step_round_trip(step: MacroStep) {
        let m = Macro {
            index: 0,
            steps: vec![step],
        };
        let serialized = m.serialize();
        let deserialized = deserialize_single(0, &serialized).unwrap();
        assert_eq!(m.steps.len(), deserialized.steps.len());
        // Can't do assert_eq! because of partialeq missing
    }

    #[test]
    fn test_step_serde_round_trip() {
        step_round_trip(MacroStep::Tap(name_to_qid(&"KC_A".to_string(), 6).unwrap()));
        step_round_trip(MacroStep::Down(name_to_qid(&"KC_B".to_string(), 6).unwrap()));
        step_round_trip(MacroStep::Up(name_to_qid(&"KC_C".to_string(), 6).unwrap()));
        step_round_trip(MacroStep::Tap(name_to_qid(&"KC_LCTL".to_string(), 6).unwrap()));
        step_round_trip(MacroStep::Tap(
            name_to_qid(&"LCTL(KC_C)".to_string(), 6).unwrap(),
        ));
        step_round_trip(MacroStep::Delay(100));
        step_round_trip(MacroStep::Text("Hello".to_string()));
    }

    #[test]
    fn test_macrostep_from_string_invalid() {
        assert!(
            MacroStep::from_string("Tap(KC_A", 6).is_err(),
            "Missing closing parenthesis"
        );
        assert!(
            MacroStep::from_string("Unknown(KC_A)", 6).is_err(),
            "Unknown macro type"
        );
        assert!(
            MacroStep::from_string("Delay(abc)", 6).is_err(),
            "Non-numeric delay"
        );
    }

    #[test]
    fn test_macrostep_from_json_invalid() {
        assert!(MacroStep::from_json(&json!("tap"), 6).is_err(), "Not an array");
        assert!(
            MacroStep::from_json(&json!(["tap"]), 6).is_err(),
            "Array too short"
        );
        assert!(
            MacroStep::from_json(&json!([123, "KC_A"]), 6).is_err(),
            "Action not a string"
        );
        assert!(
            MacroStep::from_json(&json!(["unknown", "KC_A"]), 6).is_err(),
            "Unknown action"
        );
        assert!(
            MacroStep::from_json(&json!(["delay", "abc"]), 6).is_err(),
            "Delay arg not a number"
        );
        assert!(
            MacroStep::from_json(&json!(["text", 123]), 6).is_err(),
            "Text arg not a string"
        );
        assert!(
            MacroStep::from_json(&json!(["tap", "INVALID"]), 6).is_err(),
            "Invalid keycode"
        );
    }

    #[test]
    fn test_display_macro_step() {
        assert_eq!(
            format!(
                "{}",
                MacroStep::Tap(name_to_qid(&"KC_A".to_string(), 6).unwrap())
            ),
            "Tap(KC_A)"
        );
        assert_eq!(format!("{}", MacroStep::Delay(100)), "Delay(100)");
        assert_eq!(
            format!("{}", MacroStep::Text("hello".to_string())),
            "Text(hello)"
        );
    }

    #[test]
    fn test_display_macro() {
        let m = Macro::from_string(0, &"Tap(KC_A); Delay(100)".to_string(), 6).unwrap();
        assert_eq!(format!("{}", m), "0) Tap(KC_A); Delay(100)");
        //let empty_m = Macro::empty(1);
        //assert_eq!(format!("{}", empty_m), "1) EMPTY");
    }

    #[test]
    fn test_macro_from_string_round_trip() {
        let macro_str = "Tap(KC_A); Down(KC_LEFT_CTRL); Text(hello); Up(KC_LEFT_CTRL)".to_string();
        let m1 = Macro::from_string(0, &macro_str, 6).unwrap();
        let ser = serialize(&vec![m1]);
        let m2 = deserialize(ser).unwrap();
        assert_eq!(m2.len(), 1);
        assert_eq!(format!("{}", m2[0]), format!("0) {}", macro_str));
    }

    #[test]
    fn test_deserialize_edge_cases() {
        assert!(deserialize(vec![]).unwrap().is_empty(), "Empty input");
        assert!(deserialize(vec![0]).unwrap().is_empty(), "Single zero byte");
        let m = deserialize(vec![1, 1, 4, 0, 1, 1, 5, 0]).unwrap();
        assert_eq!(m.len(), 2, "Two macros");
        assert_eq!(format!("{}", m[0]), "0) Tap(KC_A)");
        assert_eq!(format!("{}", m[1]), "1) Tap(KC_B)");
    }

    #[test]
    fn test_json_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let original_macro =
            Macro::from_string(0, &"Tap(KC_A); Delay(100); Text(test)".to_string(), 6)?;
        let macros_vec = vec![original_macro];
        let json_val = macros_to_json(&macros_vec, 6)?;
        let loaded_macros = load_macros_from_json(&serde_json::Value::Array(json_val), 6)?;
        assert_eq!(loaded_macros.len(), 1);
        assert_eq!(
            format!("{}", loaded_macros[0]),
            format!("{}", macros_vec[0])
        );
        Ok(())
    }

    #[test]
    fn test_load_macros_from_json() {
        let json = json!([
            [["tap", "KC_A", "KC_B"], ["delay", 100], ["text", "hello"]],
            [["down", "KC_LCTL"], ["tap", "KC_C"], ["up", "KC_LCTL"]]
        ]);
        let macros = load_macros_from_json(&json, 6).unwrap();
        assert_eq!(macros.len(), 2);
        assert_eq!(macros[0].steps.len(), 4); // tap, tap, delay, text
        assert_eq!(macros[1].steps.len(), 3);
        assert_eq!(
            format!("{}", macros[0]),
            "0) Tap(KC_A); Tap(KC_B); Delay(100); Text(hello)"
        );
    }
}
