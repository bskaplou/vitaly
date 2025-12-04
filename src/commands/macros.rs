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

    if capabilities.macro_count == 0 {
        return Err(common::CommandError(
            format!("device {:?} has doesn't support macros", device).to_string(),
        )
        .into());
    }
    let n: u8 = match number {
        Some(num) => {
            if num >= capabilities.macro_count {
                return Err(common::CommandError(
                    format!("Only {} macros avialable", capabilities.macro_count).to_string(),
                )
                .into());
            }
            num
        }
        None => 0,
    };
    let mut macros = protocol::load_macros(
        &dev,
        capabilities.macro_count,
        capabilities.macro_buffer_size,
    )?;
    match value {
        None => {
            if number.is_none() {
                println!("Macros list:");
                for m in macros {
                    println!("{}", m)
                }
            } else if macros.len() > n.into() {
                println!("{}", macros[n as usize])
            } else {
                return Err(common::CommandError(
                    format!("Macro {} is not defined", n).to_string(),
                )
                .into());
            }
        }
        Some(value) => {
            let m = protocol::Macro::from_string(n, value)?;
            if !m.is_empty() {
                if (n as usize) < macros.len() {
                    macros[n as usize] = m;
                } else {
                    macros.push(m);
                }
            } else if (n as usize) < macros.len() {
                macros.remove(n as usize);
            } else {
                return Err(common::CommandError(
                    format!("Can't delete undefined macro {}", n).to_string(),
                )
                .into());
            }
            println!("Updated macros list:");
            for m in &macros {
                println!("{}", m)
            }
            if capabilities.vial_version > 0 {
                let status = protocol::get_locked_status(&dev)?;
                if status.locked {
                    return Err(common::CommandError("Keyboard is locked, macroses can't be updated, keyboard might be unlocked with subcommand 'lock -u'".to_string()).into());
                }
            }
            protocol::set_macros(&dev, &capabilities, &macros)?;
            println!("Macros successfully updated");
        }
    }
    Ok(())
}
