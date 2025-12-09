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
    if capabilities.tap_dance_count == 0 {
        return Err(common::CommandError(
            format!("device {:?} has doesn't support tap dance", device).to_string(),
        )
        .into());
    }
    let n: u8;
    match number {
        Some(num) => {
            n = num;
            if n >= capabilities.tap_dance_count {
                return Err(common::CommandError(
                    format!("Only {} tap dances avialable", capabilities.tap_dance_count)
                        .to_string(),
                )
                .into());
            }
        }
        None => n = 0,
    }
    match value {
        None => {
            let tapdances = protocol::load_tap_dances(&dev, capabilities.tap_dance_count)?;
            if number.is_none() {
                let tapdance_count = tapdances.len();
                let mut first_empty = capabilities.tap_dance_count;
                for idxm in 1..=tapdance_count {
                    let idx = tapdance_count - idxm;
                    if !tapdances[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("TapDance list:");
                for idx in 0..first_empty {
                    tapdances[idx as usize].dump(capabilities.vial_version)?;
                    println!();
                }
                if first_empty < capabilities.tap_dance_count {
                    println!(
                        "TapDance slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.tap_dance_count - 1
                    );
                }
            } else {
                tapdances[n as usize].dump(capabilities.vial_version)?;
                println!();
            }
        }
        Some(value) => {
            let tapdance = match value.len() {
                0 => protocol::TapDance::empty(n),
                _ => protocol::TapDance::from_string(n, value, capabilities.vial_version)?,
            };
            protocol::set_tap_dance(&dev, &tapdance)?;
            println!("TapDance {} saved", tapdance.index);
        }
    }
    Ok(())
}
