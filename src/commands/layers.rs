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
    layout_options: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let options = if let Some(layout_options) = layout_options {
        let mut via_options = Vec::new();
        for group in layout_options.split(';') {
            if let Some((l, r)) = group.split_once(",") {
                let group: u8 = l.parse()?;
                let variant: u8 = r.parse()?;
                via_options.push((group, variant));
            }
        }
        //println!("{:?}", via_options);
        let layout_options = &meta["layouts"]["labels"];
        let mut options = protocol::LayoutOptions::from_json(0, layout_options)?;
        options.set_via_options(via_options)?;
        options
    } else {
        let layout_options = &meta["layouts"]["labels"];
        let state = protocol::load_layout_options(&dev)?;
        protocol::LayoutOptions::from_json(state, layout_options)?
    };
    //println!("{:?}", &options);
    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"], &options)?;
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
        let mut encoders = Vec::new();
        for button in &buttons {
            if button.encoder && button.wire_y == 1 {
                let e = protocol::load_encoder(&dev, layer_number, button.wire_x)?;
                encoders.push(e);
            }
        }
        encoders.sort_by(|e1, e2| e1.index.cmp(&e2.index));
        common::render_layer(&keys, &encoders, &buttons, layer_number, capabilities.vial_version)?
    }
    Ok(())
}
