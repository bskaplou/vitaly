use crate::keycodes;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    layer: u8,
    position: &str,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let clean_position = position.replace(" ", "");
    let (index, direction) = clean_position
        .split_once(",")
        .ok_or("positons should be in format index,direction")?;
    let (index, direction): (u8, u8) = (index.parse()?, direction.parse()?);

    match value {
        Some(value) => {
            if direction > 1 {
                return Err(protocol::ProtocolError::General(
                    "direction should be 0 or 1".to_string(),
                )
                .into());
            }
            let keycode = keycodes::name_to_qid(value, capabilities.vial_version)?;
            protocol::set_encoder(&dev, layer, index, direction, keycode)?;
            println!(
                "Encoder on layer={:?}, index={:?}, direction={:?} set to => {}, keycode = {:#x}",
                layer, index, direction, value, keycode,
            );
        }
        None => {
            let e = protocol::load_encoder(&dev, layer, index)?;
            let value = match direction {
                0 => keycodes::qid_to_name(e.ccw, capabilities.vial_version),
                1 => keycodes::qid_to_name(e.cw, capabilities.vial_version),
                _ => {
                    return Err(protocol::ProtocolError::General(
                        "direction should be 0 or 1".to_string(),
                    )
                    .into());
                }
            };

            println!(
                "Encoder on layer={:?}, index={:?}, direction={:?} => {}",
                layer, index, direction, value
            );
        }
    }
    Ok(())
}
