use crate::protocol::keycodes;
use crate::protocol::{
    send_recv, Capabilities, ProtocolError, BUFFER_FETCH_CHUNK, CMD_VIA_MACRO_GET_BUFFER,
    CMD_VIA_MACRO_SET_BUFFER, MESSAGE_LENGTH, VIA_UNHANDLED,
};
use hidapi::HidDevice;
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
                    let c;
                    if kc % 256 == 0 {
                        c = 0xFF00 | (kc >> 8);
                    } else {
                        c = *kc;
                    }
                    result.push((c & 0xFF) as u8);
                    result.push(((c >> 8) & 0xFF) as u8);
                }
            }
        }
        result
    }

    fn from_string(step: &str) -> Result<MacroStep, Box<dyn std::error::Error>> {
        let (left, right) = step.split_once("(").ok_or("Lack of parenthesis")?;
        let right = right[0..(right.len() - 1)].to_string();
        match left {
            "Delay" => Ok(MacroStep::Delay(right.parse()?)),
            "Text" => Ok(MacroStep::Text(right)),
            "Tap" => Ok(MacroStep::Tap(keycodes::name_to_qid(&right)?)),
            "Down" => Ok(MacroStep::Down(keycodes::name_to_qid(&right)?)),
            "Up" => Ok(MacroStep::Up(keycodes::name_to_qid(&right)?)),
            _ => {
                return Err(
                    MacroParsingError(format!("Unknown macro step {}", right).to_string()).into(),
                )
            }
        }
    }
}

impl fmt::Display for MacroStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MacroStep::Tap(kc) => Ok(write!(f, "Tap({})", keycodes::qid_to_name(*kc))?),
            MacroStep::Down(kc) => Ok(write!(f, "Down({})", keycodes::qid_to_name(*kc))?),
            MacroStep::Up(kc) => Ok(write!(f, "Up({})", keycodes::qid_to_name(*kc))?),
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

    pub fn empty(&self) -> bool {
        self.steps.len() == 0
    }

    pub fn from_strings(index: u8, steps: Vec<&str>) -> Result<Macro, Box<dyn std::error::Error>> {
        let mut parsed_steps = Vec::new();
        for step in steps {
            if step.len() > 0 {
                parsed_steps.push(MacroStep::from_string(step)?)
            }
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
        if self.empty() {
            Ok(write!(f, "EMPTY")?)
        } else {
            for (i, step) in self.steps.iter().enumerate() {
                if i > 0 {
                    write!(f, "; ")?
                }
                write!(f, "{}", step)?
            }
            return Ok(());
        }
    }
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
                            .into())
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
                        .into())
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
    if data.len() != 0 && !(data.len() == 1 && data[0] == 0) {
        for i in 0..data.len() {
            if data[i] == 0 {
                let macro_bytes = data.get(start..i).unwrap();
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

        match send_recv(&device, &[CMD_VIA_MACRO_GET_BUFFER, l1, l2, read_size]) {
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
            Err(e) => return Err(e.into()),
        }
    }
    Ok(deserialize(macro_buffer)?)
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
    //println!(">>{:?}", data);
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
        match send_recv(&device, &msg) {
            Ok(buff) => {
                if buff[0] == VIA_UNHANDLED {
                    return Err(ProtocolError::ViaUnhandledError.into());
                }
                // Fine!
            }
            Err(e) => return Err(e.into()),
        }
        offset += to_send as u16;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde1() {
        let b1: &[u8] = &[1, 2, 14, 1, 2, 206, 1, 3, 206, 116, 101, 115, 116, 0];
        assert_eq!(b1, serialize(deserialize(b1.to_vec()).unwrap()));
    }

    #[test]
    fn test_serde2() {
        let b1: &[u8] = &[
            1, 4, 6, 1, 1, 4, 236, 4, 0, 116, 101, 115, 116, 0, 1, 2, 30, 0, 1, 3, 30, 0, 1, 1, 30,
            0, 84, 69, 83, 84, 1, 4, 101, 1, 0, 1, 5, 126, 255, 0,
        ];
        assert_eq!(b1, serialize(deserialize(b1.to_vec()).unwrap()));
    }
}
