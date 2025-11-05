pub mod buffer;

use buffer::Buffer;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("MetaParsingError")]
pub struct MetaParsingError;

#[derive(Debug)]
pub struct Button {
    pub x: f64,
    pub y: f64,
    pub h: f64,
    pub w: f64,
    pub wire_x: u8,
    pub wire_y: u8,
}

impl Button {
    pub fn scale(&self, scale: f64) -> Button {
        Button {
            x: self.x * scale,
            y: self.y * scale,
            h: self.h * scale,
            w: self.w * scale,
            wire_x: self.wire_x,
            wire_y: self.wire_y,
        }
    }
}

pub fn keymap_to_buttons(keymap: &Value) -> Result<Vec<Button>, Box<dyn std::error::Error>> {
    let mut buttons = Vec::new();
    let mut x_pos = 0f64;
    let mut y_pos = 0f64;
    let mut x_mod = 0f64;
    let y_mod = 0f64;
    let mut rx = 0f64;
    let mut ry = 0f64;
    match keymap.as_array() {
        Some(rows) => {
            let mut w = 1f64;
            let mut h = 1f64;
            let mut r = 0f64;
            //let mut y = 0f64;
            //let mut x = 0f64;
            let mut decal = false;

            for row in rows.iter() {
                //print!("{:?}", row);
                match row.as_array() {
                    Some(items) => {
                        for item in items {
                            if item.is_object() {
                                for (key, value) in item.as_object().unwrap() {
                                    match key.as_str() {
                                        /*
                                        "x" => {
                                            x = value.as_f64().unwrap();
                                            x_mod += value.as_f64().unwrap();
                                        }
                                        "y" => {
                                            y_mod += value.as_f64().unwrap();
                                            y = value.as_f64().unwrap();
                                        }
                                        */
                                        "w" => w = value.as_f64().unwrap(),
                                        "h" => h = value.as_f64().unwrap(),
                                        "r" => r = value.as_f64().unwrap(),
                                        "rx" => rx = value.as_f64().unwrap(),
                                        "ry" => ry = value.as_f64().unwrap(),
                                        "d" => decal = value.as_bool().unwrap(),
                                        &_ => {
                                            // println!("warning ignored value {:?} = {:?}", key, value)
                                        }
                                    }
                                }
                            } else {
                                if !decal {
                                    // skip decals entirely
                                    let mut value = item.as_str().unwrap();
                                    if value.contains('\n') {
                                        let mut s = value.split('\n');
                                        loop {
                                            value = s.next().unwrap();
                                            if value.len() > 0 {
                                                // println!("value {:?}", value);
                                                break;
                                            }
                                        }
                                    }
                                    let mut parts = value.split(',');
                                    let xx: u8 = parts.next().unwrap().parse().unwrap();
                                    let yy: u8 = parts.next().unwrap().parse().unwrap();
                                    let but;
                                    if r == 0.0 && rx == 0.0 && ry == 0.0 {
                                        let bx = x_pos + x_mod;
                                        let by = y_pos + y_mod;
                                        let bw = w;
                                        let bh = h;
                                        but = Button {
                                            x: bx,
                                            y: by,
                                            w: bw,
                                            h: bh,
                                            wire_x: xx,
                                            wire_y: yy,
                                        };
                                    } else {
                                        //println!("p = {},{}, r = {:?}, rx = {:?}, ry = {:?}, x = {:?}, y = {:?}", xx, yy, r, rx, ry, x, y);
                                        /*
                                        let teta = -r.to_radians();
                                        let teta_sin = teta.sin();
                                        let teta_cos = teta.cos();
                                        let bx = x * teta_cos + y * teta_sin + rx;
                                        let by = -x * teta_sin + y * teta_cos + ry;
                                        let bw = 1.0;
                                        let bh = 1.0;
                                        but = Button {
                                            x: bx,
                                            y: by,
                                            w: bw,
                                            h: bh,
                                            wire_x: xx,
                                            wire_y: yy,
                                        };
                                        */
                                        return Err(MetaParsingError.into());
                                    }
                                    buttons.push(but);
                                }
                                //println!("! {:?} => {:?}", item.as_str().unwrap(), &but);
                                x_pos += x_mod + w;
                                w = 1.0;
                                h = 1.0;
                                x_mod = 0.0;
                                decal = false;
                            }
                        }
                    }
                    None => {
                        // sometimes first element is dict
                        // return Err(MetaParsingError);
                    }
                }
                x_pos = 0.0;
                y_pos += 1.0;
                //r = 0.0;
            }
        }
        None => return Err(MetaParsingError.into()),
    }
    Ok(buttons)
}

pub fn render_and_dump(buttons: Vec<Button>, labels: Option<HashMap<(u8, u8), String>>) {
    let mut buff = Buffer::new();
    for button in buttons {
        let b = button.scale(4.0);
        let lu = (b.x as usize, b.y as usize);
        let ru = ((b.x + b.w - 1.0) as usize, b.y as usize);
        let lb = (b.x as usize, (b.y + b.h - 1.0) as usize);
        let rb = ((b.x + b.w - 1.0) as usize, (b.y + b.h - 1.0) as usize);
        buff.put(lu.0, lu.1, '╔');
        for x in (lu.0 + 1)..ru.0 {
            buff.put(x, lu.1, '═');
        }
        buff.put(ru.0, ru.1, '╗');
        for y in (lu.1 + 1)..lb.1 {
            buff.put(lu.0, y, '║');
        }
        for y in (ru.1 + 1)..rb.1 {
            buff.put(ru.0, y, '║');
        }
        buff.put(lb.0, lb.1, '╚');
        for x in (lb.0 + 1)..rb.0 {
            buff.put(x, lb.1, '═');
        }
        buff.put(rb.0, rb.1, '╝');

        match labels {
            Some(ref labels) => match labels.get(&(button.wire_x, button.wire_y)) {
                Some(label) => {
                    // FIXME comma treatment is too ugly :( but works
                    let mut we_got_comma = false;
                    for (line, chunk) in label.split(',').enumerate() {
                        if chunk.len() == 0 {
                            if !we_got_comma {
                                buff.put(lu.0 + 1 + line, lu.1 + 1, ',');
                                we_got_comma = true;
                            }
                        } else {
                            for (i, c) in chunk.chars().enumerate() {
                                buff.put(lu.0 + 1 + i, lu.1 + 1 + line, c);
                            }
                        }
                    }
                }
                None => {
                    // No label => empty button
                }
            },
            None => {
                let xx = format!("{}", button.wire_x);
                let yy = format!("{}", button.wire_y);
                for (i, c) in xx.chars().enumerate() {
                    buff.put(lu.0 + 1 + i, lu.1 + 1, c);
                }
                for (i, c) in yy.chars().enumerate() {
                    buff.put(lu.0 + 1 + i, lu.1 + 2, c);
                }
            }
        }
    }
    buff.dump();
}
