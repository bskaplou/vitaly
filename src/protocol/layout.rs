use crate::protocol::{
    CMD_VIA_GET_KEYBOARD_VALUE, CMD_VIA_SET_KEYBOARD_VALUE, ProtocolError, VIA_LAYOUT_OPTIONS,
    VIA_UNHANDLED, send_recv,
};
use hidapi::HidDevice;
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
pub struct LayoutOptions<'a> {
    pub state: u32,
    pub options: Vec<(&'a str, Vec<&'a str>, u8)>,
}

impl LayoutOptions<'_> {
    pub fn empty() -> LayoutOptions<'static> {
        LayoutOptions {
            state: 0,
            options: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.options.len() == 0
    }

    pub fn from_json(
        state: u32,
        labels: &Value,
    ) -> Result<LayoutOptions<'_>, Box<dyn std::error::Error>> {
        let mut options = Vec::new();
        let mut start_bit: u8 = 0;
        if matches!(labels, Value::Null) {
            return Ok(LayoutOptions::empty());
        }
        for label in labels
            .as_array()
            .ok_or("layout/labels should be an array")?
            .iter()
            .rev()
        {
            match label {
                Value::String(name) => {
                    options.push((name.as_str(), vec!["false", "true"], start_bit));
                    start_bit += 1;
                }
                Value::Array(variants) => {
                    let mut vars = Vec::new();
                    for variant in &variants[1..] {
                        vars.push(
                            variant
                                .as_str()
                                .ok_or("array layout/labels should be array of strings")?,
                        )
                    }
                    options.push((
                        variants[0]
                            .as_str()
                            .ok_or("layout/label name should be string")?,
                        vars,
                        start_bit,
                    ));
                    start_bit += variants.len() as u8 - 2;
                }
                _ => {
                    return Err(ProtocolError::General(
                        "labels should be string or array of strings".to_string(),
                    )
                    .into());
                }
            }
        }
        options.reverse();
        Ok(LayoutOptions { state, options })
    }

    pub fn via_options(&self) -> Vec<(u8, u8)> {
        let mut result = Vec::new();
        for (option_idx, (_, variants, start_bit)) in self.options.iter().enumerate() {
            // nullify other options bits and put current option bits to rightmost position
            let ignore_high_bits = 33 - start_bit - variants.len() as u8;
            let variant_bits = self.state.overflowing_shl(ignore_high_bits.into()).0.overflowing_shr((start_bit + ignore_high_bits).into()).0;
            for (variant_idx, _) in variants.iter().enumerate() {
                // zero means default option, other bit positon
                if (variant_bits == 0 && variant_idx == 0)
                    || (variant_idx > 0 && variant_bits >> (variant_idx - 1) == 1)
                {
                    result.push((option_idx as u8, variant_idx as u8));
                }
            }
        }
        result
    }

    pub fn set_via_options(
        &mut self,
        options: Vec<(u8, u8)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (option_idx, (_, variants, start_bit)) in self.options.iter().enumerate() {
            for (new_option, new_variant) in &options {
                if option_idx as u8 == *new_option {
                    let ignore_high_bits = 33 - start_bit - variants.len() as u8;
                    let mask = !((0xFFFFFFFF << ignore_high_bits)
                        >> (start_bit + ignore_high_bits)
                        << start_bit);
                    let variant_bit = if *new_variant == 0 {
                        0
                    } else {
                        1 << (new_variant - 1 + start_bit)
                    };
                    self.state = self.state & mask | variant_bit;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for LayoutOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let options = self.via_options();
        for (option_idx, (name, variants, _)) in self.options.iter().enumerate() {
            writeln!(f, "{}) {}:", option_idx, name)?;
            for (variant_idx, variant) in variants.iter().enumerate() {
                if options[option_idx] == (option_idx as u8, variant_idx as u8) {
                    writeln!(f, "\t{}) {} <= currently selected", variant_idx, variant)?;
                } else {
                    writeln!(f, "\t{}) {}", variant_idx, variant)?;
                }
            }
        }
        Ok(())
    }
}

pub fn load_layout_options(device: &HidDevice) -> Result<u32, Box<dyn std::error::Error>> {
    match send_recv(device, &[CMD_VIA_GET_KEYBOARD_VALUE, VIA_LAYOUT_OPTIONS]) {
        Ok(data) => {
            if data[0] != VIA_UNHANDLED {
                let options = ((data[2] as u32) << 24)
                    + ((data[3] as u32) << 16)
                    + ((data[4] as u32) << 8)
                    + (data[5] as u32);
                Ok(options)
            } else {
                Err(ProtocolError::ViaUnhandledError.into())
            }
        }
        Err(e) => Err(e),
    }
}

pub fn set_layout_options(
    device: &HidDevice,
    options: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    match send_recv(
        device,
        &[
            CMD_VIA_SET_KEYBOARD_VALUE,
            VIA_LAYOUT_OPTIONS,
            ((options >> 24) & 0xFF) as u8,
            ((options >> 16) & 0xFF) as u8,
            ((options >> 8) & 0xFF) as u8,
            (options & 0xFF) as u8,
        ],
    ) {
        Ok(data) => {
            if data[0] != VIA_UNHANDLED {
                Ok(())
            } else {
                Err(ProtocolError::ViaUnhandledError.into())
            }
        }
        Err(e) => Err(e),
    }
}
