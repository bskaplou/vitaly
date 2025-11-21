use std::fmt;

use crate::protocol::{
    CMD_VIA_LIGHTING_GET_VALUE, CMD_VIA_LIGHTING_SET_VALUE, VIA_UNHANDLED, VIALRGB_GET_INFO,
    VIALRGB_GET_MODE, VIALRGB_GET_SUPPORTED, VIALRGB_SET_MODE, send_recv,
};
use hidapi::HidDevice;

#[derive(Debug)]
pub struct RGBInfo {
    pub version: u16,
    pub effect: u16,
    pub effect_speed: u8,
    pub color_h: u8,
    pub color_s: u8,
    pub color_v: u8,
    pub max_brightness: u8,
    pub effects: Vec<u16>,
}

impl RGBInfo {
    pub fn effect_name(id: u16) -> Result<&'static str, Box<dyn std::error::Error>> {
        match id {
            0 => Ok("Disable"),
            1 => Ok("Direct Control"),
            2 => Ok("Solid Color"),
            3 => Ok("Alphas Mods"),
            4 => Ok("Gradient Up Down"),
            5 => Ok("Gradient Left Right"),
            6 => Ok("Breathing"),
            7 => Ok("Band Sat"),
            8 => Ok("Band Val"),
            9 => Ok("Band Pinwheel Sat"),
            10 => Ok("Band Pinwheel Val"),
            11 => Ok("Band Spiral Sat"),
            12 => Ok("Band Spiral Val"),
            13 => Ok("Cycle All"),
            14 => Ok("Cycle Left Right"),
            15 => Ok("Cycle Up Down"),
            16 => Ok("Rainbow Moving Chevron"),
            17 => Ok("Cycle Out In"),
            18 => Ok("Cycle Out In Dual"),
            19 => Ok("Cycle Pinwheel"),
            20 => Ok("Cycle Spiral"),
            21 => Ok("Dual Beacon"),
            22 => Ok("Rainbow Beacon"),
            23 => Ok("Rainbow Pinwheels"),
            24 => Ok("Raindrops"),
            25 => Ok("Jellybean Raindrops"),
            26 => Ok("Hue Breathing"),
            27 => Ok("Hue Pendulum"),
            28 => Ok("Hue Wave"),
            29 => Ok("Typing Heatmap"),
            30 => Ok("Digital Rain"),
            31 => Ok("Solid Reactive Simple"),
            32 => Ok("Solid Reactive"),
            33 => Ok("Solid Reactive Wide"),
            34 => Ok("Solid Reactive Multiwide"),
            35 => Ok("Solid Reactive Cross"),
            36 => Ok("Solid Reactive Multicross"),
            37 => Ok("Solid Reactive Nexus"),
            38 => Ok("Solid Reactive Multinexus"),
            39 => Ok("Splash"),
            40 => Ok("Multisplash"),
            41 => Ok("Solid Splash"),
            42 => Ok("Solid Multisplash"),
            43 => Ok("Pixel Rain"),
            44 => Ok("Pixel Fractal"),
            _ => Err("no such effect".into()),
        }
    }
}

impl fmt::Display for RGBInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(
            f,
            "RGB verions: {}, max_brightness: {}",
            self.version, self.max_brightness
        )?;
        writeln!(f, "supported_effects:")?;
        for effect in &self.effects {
            if let Ok(name) = RGBInfo::effect_name(*effect) {
                writeln!(f, "\t{}) {}", effect, name)?;
            };
        }
        write!(f, "\ncurrent settings:\n")?;
        if let Ok(name) = RGBInfo::effect_name(self.effect) {
            writeln!(f, "\teffect: {} - {}", self.effect, name)?;
        };
        writeln!(f, "\teffect_speed: {}", self.effect_speed)?;
        writeln!(
            f,
            "\tcolor_hsv: (h={}, s={}, v={})",
            self.color_h, self.color_s, self.color_v
        )?;
        Ok(())
    }
}

pub fn load_rgb_info(device: &HidDevice) -> Result<RGBInfo, Box<dyn std::error::Error>> {
    let version: u16;
    let max_brightness: u8;
    let mut effect: u16 = 0;
    let mut effect_speed: u8 = 0;
    let mut color_h: u8 = 0;
    let mut color_s: u8 = 0;
    let mut color_v: u8 = 0;

    let mut effects: Vec<u16> = Vec::new();
    effects.push(0);

    match send_recv(device, &[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_INFO]) {
        Ok(data) => {
            if data[0] != VIA_UNHANDLED {
                version = (data[2] as u16) + ((data[3] as u16) << 8);
                max_brightness = data[4];
                let mut effect: u16 = 0;
                'top: loop {
                    let e2 = (effect >> 8 & 0xFF) as u8;
                    let e1 = (effect & 0xFF) as u8;

                    match send_recv(
                        device,
                        &[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_SUPPORTED, e1, e2],
                    ) {
                        Ok(data) => {
                            for i in 0..15 {
                                effect = (data[i * 2 + 2] as u16) + ((data[i * 2 + 3] as u16) << 8);
                                if effect == 0xFFFF {
                                    break 'top;
                                }
                                effects.push(effect);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            } else {
                version = 0;
                max_brightness = 0;
            }
        }
        Err(e) => return Err(e),
    }

    if version == 1 {
        match send_recv(device, &[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_MODE]) {
            Ok(data) => {
                effect = (data[2] as u16) + ((data[3] as u16) << 8);
                effect_speed = data[4];
                color_h = data[5];
                color_s = data[6];
                color_v = data[7];
            }
            Err(e) => return Err(e),
        }
    }

    Ok(RGBInfo {
        version,
        max_brightness,
        effects,
        effect,
        effect_speed,
        color_h,
        color_s,
        color_v,
    })
}

pub fn set_rgb_info(
    device: &HidDevice,
    rgb_info: &RGBInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let e1 = (rgb_info.effect & 0xFF) as u8;
    let e2 = ((rgb_info.effect >> 8) & 0xFF) as u8;
    match send_recv(
        device,
        &[
            CMD_VIA_LIGHTING_SET_VALUE,
            VIALRGB_SET_MODE,
            e1,
            e2,
            rgb_info.effect_speed,
            rgb_info.color_h,
            rgb_info.color_s,
            rgb_info.color_v,
        ],
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(())
}
