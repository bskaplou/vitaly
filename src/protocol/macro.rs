use crate::protocol::{
    send, send_recv, ProtocolError, BUFFER_FETCH_CHUNK, CMD_VIA_MACRO_GET_BUFFER,
    CMD_VIA_MACRO_SET_BUFFER, VIA_UNHANDLED,
};
use hidapi::HidDevice;
use std::cmp::min;
use thiserror::Error;

const SS_QMK_PREFIX: u8 = 1;
const SS_TAP_CODE: u8 = 1;
const SS_DOWN_CODE: u8 = 2;
const SS_UP_CODE: u8 = 3;
const SS_DELAY_CODE: u8 = 4;
const VIAL_MACRO_EXT_TAP: u8 = 5;
const VIAL_MACRO_EXT_DOWN: u8 = 6;
const VIAL_MACRO_EXT_UP: u8 = 7;

#[derive(Debug)]
pub enum MacroStep {
    Tap(u16),
    Down(u16),
    Up(u16),
    Delay(u16),
    Text(String),
}

#[derive(Debug)]
pub struct Macro {
    pub index: u8,
    pub steps: Vec<MacroStep>,
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct MacroParsingError(String);

// State machine here
enum MacroParsingState {
    Start,
    Text(usize),
    NextCommand,
    Command(u8),
    CommandWithArgs(u8, u8),
}

impl Macro {}

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

pub fn load_macros(
    device: &HidDevice,
    count: u8,
    buffer_size: u16,
) -> Result<Vec<Macro>, Box<dyn std::error::Error>> {
    let mut offset: u16 = 0;
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
}
