use crate::protocol;
use argh::FromArgs;
use hidapi::{DeviceInfo, HidApi};

#[derive(FromArgs, PartialEq, Debug)]
/// RGB lighting
#[argh(subcommand, name = "rgb")]
pub struct CommandRgb {
    /// show info
    #[argh(switch, short = 'i')]
    pub info: bool,

    /// set effect - effect id from supported list
    #[argh(option, short = 'e')]
    pub effect: Option<u16>,

    /// set effect speed 0-255
    #[argh(option, short = 's')]
    pub speed: Option<u8>,

    /// set brightness 0-max_brightness
    #[argh(option, short = 'b')]
    pub brightness: Option<u8>,

    /// set color for example #ffffff
    #[argh(option, short = 'c')]
    pub color: Option<String>,

    /// persist rgb config to keep settings after restart
    #[argh(switch, short = 'p')]
    pub persist: bool,

    /// list supported effects
    #[argh(switch, short = 'l')]
    pub list: bool,

    /// direct led control, NB works only if effect = 1, example value 0-5=#ffffff;6=#0000ff
    #[argh(option, short = 'd')]
    pub direct: Option<String>,
}

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    cmd: &CommandRgb,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;

    let mut rgb_info = protocol::load_rgb_info(&dev)?;
    let mut update = false;
    if let Some(s) = cmd.speed {
        rgb_info.effect_speed = s;
        update = true;
    }
    if let Some(e) = cmd.effect {
        rgb_info.effect = e;
        update = true;
    }
    if let Some(c) = &cmd.color {
        rgb_info.set_color(c)?;
        update = true;
    }
    // brightness applied after color, because it overwrites it hsv
    if let Some(b) = cmd.brightness {
        if b > rgb_info.max_brightness {
            rgb_info.color_v = rgb_info.max_brightness;
        } else {
            rgb_info.color_v = b;
        }
        update = true;
    }
    if cmd.list {
        rgb_info.dump_supported_effects();
    }
    if cmd.info {
        println!("\n{}", rgb_info);
    }
    if update {
        protocol::set_rgb_mode(&dev, &rgb_info)?;
        println!("RGB settings updated...");
    }
    if cmd.persist {
        protocol::persist_rgb(&dev)?;
        println!("RGB settings persisted...");
    }
    if let Some(command) = &cmd.direct {
        protocol::set_leds_direct(&dev, command, rgb_info.max_brightness)?;
    }
    Ok(())
}
