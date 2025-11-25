use crate::common;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    number: Option<u8>,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    if capabilities.alt_repeat_key_count == 0 {
        return Err(common::CommandError(
            format!("device {:?} has doesn't support alt repeat keys", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.alt_repeat_key_count {
                return Err(common::CommandError(
                    format!(
                        "Only {} alt repleat keys avialable",
                        capabilities.alt_repeat_key_count
                    )
                    .to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    match value {
        Some(value) => {
            let alt_repeat = match value.len() {
                0 => protocol::AltRepeat::empty(n),
                _ => protocol::AltRepeat::from_string(n, value)?,
            };
            protocol::set_alt_repeat(&dev, &alt_repeat)?;
            println!("AltRepeat {} saved", alt_repeat);
        }
        None => {
            let altrepeats = protocol::load_alt_repeats(&dev, capabilities.alt_repeat_key_count)?;
            if number.is_none() {
                let altrepeat_count = altrepeats.len();
                let mut first_empty = capabilities.alt_repeat_key_count;
                for idxm in 1..=altrepeat_count {
                    let idx = altrepeat_count - idxm;
                    if !altrepeats[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("AltRepeat list:");
                for idx in 0..first_empty {
                    println!("{}", altrepeats[idx as usize]);
                }
                if first_empty < capabilities.alt_repeat_key_count {
                    println!(
                        "AltRepeat slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.alt_repeat_key_count - 1
                    );
                }
            } else {
                println!("{}", altrepeats[n as usize]);
            }
        }
    }
    Ok(())
}
