use crate::common;
use crate::keymap;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    positions: bool,
    number: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"])?;
    if positions {
        keymap::render_and_dump(&buttons, None);
    } else {
        let layer_number: u8 = number.unwrap_or_default();
        let cols = meta["matrix"]["cols"]
            .as_u64()
            .ok_or("matrix/cols not found in meta")? as u8;
        let rows = meta["matrix"]["rows"]
            .as_u64()
            .ok_or("matrix/rows not found in meta")? as u8;
        let keys = protocol::load_layers_keys(&dev, capabilities.layer_count, rows, cols)?;
        common::render_layer(&keys, &buttons, layer_number)?
    }
    Ok(())
}
