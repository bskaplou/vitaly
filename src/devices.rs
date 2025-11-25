use crate::protocol;
use hidapi::{DeviceInfo, HidApi};

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    capabilities: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if capabilities {
        let device_path = device.path();
        let dev = api.open_path(device_path)?;
        let capabilities = protocol::scan_capabilities(&dev)?;
        println!("Capabilities:\n\tvia_version: {}", capabilities.via_version);
        println!("\tvial_version: {}", capabilities.vial_version);
        println!(
            "\tcompanion_hid_version: {}",
            capabilities.companion_hid_version
        );
        println!("\tlayer_count: {}", capabilities.layer_count);
        println!("\tmacro_count: {}", capabilities.macro_count);
        println!("\tmacro_buffer_size: {}", capabilities.macro_buffer_size);
        println!("\ttap_dance_count: {}", capabilities.tap_dance_count);
        println!("\tcombo_count: {}", capabilities.combo_count);
        println!("\tkey_override_count: {}", capabilities.key_override_count);
        println!(
            "\talt_repeat_key_count: {}",
            capabilities.alt_repeat_key_count
        );
        println!("\tcaps_word: {}", capabilities.caps_word);
        println!("\tlayer_lock: {}", capabilities.layer_lock);
    }
    println!();
    Ok(())
}
