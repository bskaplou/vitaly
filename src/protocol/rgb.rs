use palette::FromColor;
use palette::{Hsv, Srgb};
use std::fmt;

use crate::protocol::{
    CMD_VIA_LIGHTING_GET_VALUE, CMD_VIA_LIGHTING_SAVE, CMD_VIA_LIGHTING_SET_VALUE, MESSAGE_LENGTH,
    VIA_UNHANDLED, VIALRGB_DIRECT_FASTSET, VIALRGB_GET_INFO, VIALRGB_GET_MODE,
    VIALRGB_GET_NUMBER_LEDS, VIALRGB_GET_SUPPORTED, VIALRGB_SET_MODE, send_recv,
};
use hidapi::HidDevice;

pub fn rgb_to_hsv(
    color: &str,
    max_brightness: u8,
) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
    let rgb: Srgb<u8> = color.parse()?;
    let ru8: Srgb = Srgb::from(rgb);
    let hsv: Hsv = Hsv::from_color(ru8);
    let hu8: Hsv<_, u8> = hsv.into_format::<u8>();
    let (h, s, v) = hu8.into_components();
    let normal_brightness = ((v as f64) * (max_brightness as f64) / 255.0) as u8;
    Ok((h.into_inner(), s, normal_brightness))
}

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
    pub leds_count: u16,
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

    pub fn dump_supported_effects(&self) {
        println!("supported_effects:");
        for effect in &self.effects {
            if let Ok(name) = RGBInfo::effect_name(*effect) {
                println!("{}) {}", effect, name);
            };
        }
    }

    pub fn set_color(&mut self, color: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (h, s, v) = rgb_to_hsv(color, self.max_brightness)?;
        self.color_h = h;
        self.color_s = s;
        self.color_v = v;
        Ok(())
    }
}

impl fmt::Display for RGBInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(
            f,
            "RGB verions: {}, leds_count: {}, max_brightness: {}",
            self.version, self.leds_count, self.max_brightness
        )?;
        writeln!(f, "current settings:")?;
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
    let mut leds_count: u16 = 0;

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
        match send_recv(
            device,
            &[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_NUMBER_LEDS],
        ) {
            Ok(data) => {
                leds_count = (data[2] as u16) + ((data[3] as u16) << 8);
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
        leds_count,
    })
}

pub fn set_rgb_mode(
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

pub fn persist_rgb(device: &HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    match send_recv(device, &[CMD_VIA_LIGHTING_SAVE]) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(())
}

fn set_leds_range(
    device: &HidDevice,
    from: u16,
    to: u16,
    color: &str,
    max_brightness: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let (h, s, v) = rgb_to_hsv(color, max_brightness)?;
    let mut buff: [u8; MESSAGE_LENGTH] = [0u8; MESSAGE_LENGTH];
    buff[0] = CMD_VIA_LIGHTING_SET_VALUE;
    buff[1] = VIALRGB_DIRECT_FASTSET;
    buff[2] = (from & 0xFF) as u8;
    buff[3] = (from >> 8 & 0xFF) as u8;
    buff[4] = (to - from + 1) as u8;
    for i in 0..=(to - from) {
        let start = (5 + i * 3) as usize;
        buff[start] = h;
        buff[start + 1] = s;
        buff[start + 2] = v;
    }
    //println!("{:?}", buff);
    match send_recv(device, &buff) {
        Ok(data) => {
            //println!("{:?}", data);
        }
        Err(e) => return Err(e),
    }
    //println!("{}->{} = {} = {} {} {}", from, to, color, h, s, v);
    Ok(())
}

pub fn set_leds_direct(
    device: &HidDevice,
    command: &str,
    max_brightness: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let instructions = command.replace(" ", "");
    for instruction in instructions.split(";") {
        if let Some((leds, color)) = instruction.split_once("=") {
            for led in leds.split(",") {
                if let Some((led_from, led_to)) = led.split_once("-") {
                    set_leds_range(
                        device,
                        led_from.parse()?,
                        led_to.parse()?,
                        color,
                        max_brightness,
                    )?;
                } else {
                    let led_num: u16 = led.parse()?;
                    set_leds_range(device, led_num, led_num, color, max_brightness)?;
                }
            }
        } else {
            return Err("Bad instruction".into());
        }
    }
    Ok(())
}
