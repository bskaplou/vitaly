use crate::common;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
    option: &Option<u8>,
    value: &Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let layout_options = &meta["layouts"]["labels"];
    let state = protocol::load_layout_options(&dev)?;
    let mut options = protocol::LayoutOptions::from_json(state, layout_options)?;
    match option {
        Some(o) => match value {
            Some(v) => {
                options.set_via_options(vec![(*o, *v)])?;
                protocol::set_layout_options(&dev, options.state)?;
                println!("Layout options has been updated");
            }
            None => {
                println!("Current layout options\n{}", &options);
            }
        },
        None => {
            println!("Current layout options\n{}", &options);
        }
    }
    Ok(())
}
