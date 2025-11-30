use crate::common;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    meta_file: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    let meta = common::load_meta(&dev, &capabilities, meta_file)?;
    let layout_options = &meta["layouts"]["labels"];
    let state = protocol::load_layout_options(&dev)?;
    let mut options = protocol::LayoutOptions::from_json(state, layout_options)?;
    println!("{:?}", &options);
    println!("{:?}", options.via_options());
    println!("{}", &options);
    options.set_via_options(vec!((1, 1), (2, 2)));
    println!("{:?}", &options);
    println!("{:?}", options.via_options());
    println!("{}", &options);
    Ok(())
}
