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
    if capabilities.key_override_count == 0 {
        return Err(common::CommandError(
            format!("device {:?} has doesn't support key override", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.key_override_count {
                return Err(common::CommandError(
                    format!(
                        "Only {} key overrides avialable",
                        capabilities.key_override_count
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
            let ko = match value.len() {
                0 => protocol::KeyOverride::empty(n),
                _ => protocol::KeyOverride::from_string(n, value, capabilities.vial_version)?,
            };
            protocol::set_key_override(&dev, &ko)?;
            println!("KeyOverride {} saved", ko.index);
        }
        None => {
            let keyoverrides = protocol::load_key_overrides(&dev, capabilities.key_override_count)?;
            if number.is_none() {
                let keyoverride_count = keyoverrides.len();
                let mut first_empty = capabilities.key_override_count;
                for idxm in 1..=keyoverride_count {
                    let idx = keyoverride_count - idxm;
                    if !keyoverrides[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("KeyOverride list:");
                for idx in 0..first_empty {
                    keyoverrides[idx as usize].dump(capabilities.vial_version)?;
                    println!();
                }
                if first_empty < capabilities.key_override_count {
                    println!(
                        "KeyOverride slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.key_override_count - 1
                    );
                }
            } else {
                keyoverrides[n as usize].dump(capabilities.vial_version)?;
                println!();
            }
        }
    }

    Ok(())
}
