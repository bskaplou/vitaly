use crate::common;
use crate::keycodes;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    layer: u8,
    position: &str,
    value: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or("matrix/cols not found in meta")? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or("matrix/rows not found in meta")? as u8;

    let row: u8;
    let col: u8;
    if let Some((r, c)) = position.split_once(',') {
        row = r.parse()?;
        col = c.parse()?;
    } else {
        return Err(common::CommandError("position format is 'row,col'".to_string()).into());
    }
    match value {
        Some(value) => match keycodes::name_to_qid(value) {
            Ok(keycode) => {
                protocol::set_keycode(&dev, layer, row, col, keycode)?;
                println!(
                    "Key on layer={:?}, row={:?}, col={:?} set to => {}, keycode = {:#x}",
                    layer, row, col, value, keycode,
                );
            }
            Err(e) => {
                return Err(common::CommandError(
                    format!("failed to build keycode {:?}", e).to_string(),
                )
                .into());
            }
        },
        None => {
            let keys = protocol::load_layers_keys(&dev, capabilities.layer_count, rows, cols)?;
            let label = keys.get_long(layer, row, col)?;
            println!(
                "Key on layer={:?}, row={:?}, col={:?} => {}",
                layer, row, col, label
            );
        }
    }

    Ok(())
}
