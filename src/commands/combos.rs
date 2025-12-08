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

    if capabilities.combo_count == 0 {
        return Err(common::CommandError(
            format!("device {:?} has doesn't support combos", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.combo_count {
                return Err(common::CommandError(
                    format!("Only {} combo avialable", capabilities.combo_count).to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    match value {
        None => {
            let combos = protocol::load_combos(&dev, capabilities.combo_count)?;
            if number.is_none() {
                let combo_count = combos.len();
                let mut first_empty = capabilities.combo_count;
                for idxm in 1..=combo_count {
                    let idx = combo_count - idxm;
                    if !combos[idx as usize].is_empty() {
                        break;
                    }
                    first_empty = idx as u8;
                }
                println!("Combos list:");
                for idx in 0..first_empty {
                    println!("{}", combos[idx as usize]);
                }
                if first_empty < capabilities.combo_count {
                    println!(
                        "Combo slots {} - {} are EMPTY",
                        first_empty,
                        capabilities.combo_count - 1
                    );
                }
            } else {
                println!("{}", combos[n as usize]);
            }
        }
        Some(value) => {
            let combo = match value.len() {
                0 => protocol::Combo::empty(n),
                _ => protocol::Combo::from_string(n, value, capabilities.vial_version)?,
            };
            protocol::set_combo(&dev, &combo)?;
            println!("Combo {} saved", combo);
        }
    }
    Ok(())
}
