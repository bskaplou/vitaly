use anyhow::{Result, anyhow};
pub mod buffer;

use crate::protocol;
pub use buffer::Buffer;
use palette::Srgb;
use serde_json::Value;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("MetaParsingError")]
pub struct MetaParsingError;

#[derive(Debug, Clone)]
pub struct Button {
    pub x: f64,
    pub y: f64,
    pub h: f64,
    pub w: f64,
    pub wire_x: u8,
    pub wire_y: u8,
    pub layout_options: Option<(u8, u8)>,
    pub encoder: bool,
    pub decal: bool,
    pub color: Option<(u8, u8, u8)>,
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
            layout_options: self.layout_options,
            encoder: self.encoder,
            decal: self.decal,
            color: self.color,
        }
    }
}

fn matches(options: &[(u8, u8)], option: Option<(u8, u8)>) -> bool {
    //return true;
    match option {
        Some(o) => {
            if options.len() > o.1.into() {
                options[o.0 as usize].1 == o.1
            } else {
                false
            }
        }
        None => true,
    }
}

pub fn get_encoders_count(keymap: &Value) -> Result<u8> {
    let mut result = 0;
    if let Some(rows) = keymap.as_array() {
        for row in rows.iter() {
            if let Some(items) = row.as_array() {
                for item in items {
                    if let Value::String(label) = item {
                        let parts: Vec<_> = label.split("\n").collect();
                        if parts.len() > 9
                            && parts[9].starts_with("e")
                            && let Some((index, direction)) = parts[0].split_once(",")
                            && direction == "0"
                        {
                            let index: u8 = index.parse()?;
                            result = max(result, index + 1);
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}

pub fn keymap_to_buttons(
    keymap: &Value,
    current_options: &protocol::LayoutOptions,
) -> Result<Vec<Button>> {
    let via_options = current_options.via_options();
    let mut option_groups = HashMap::<u8, (f64, f64)>::new();
    let mut buttons = Vec::new();
    let rows = keymap
        .as_array()
        .ok_or(anyhow!("keymap should be an array"))?;
    let mut x_pos = 0f64;
    let mut y_pos = 0f64;
    let mut rx = 0f64;
    let mut ry = 0f64;
    let mut w = 1f64;
    let mut h = 1f64;
    let mut r = 0f64;
    let mut decal = false;
    let mut cluster: (f64, f64) = (0.0, 0.0);
    let mut color: Option<(u8, u8, u8)> = None;

    for row in rows.iter() {
        match row.as_array() {
            Some(items) => {
                for item in items {
                    match item {
                        Value::Object(item) => {
                            for (key, value) in item {
                                match key.as_str() {
                                    "x" => {
                                        let x = value
                                            .as_f64()
                                            .ok_or(anyhow!("x should be a number"))?;
                                        x_pos += x;
                                    }
                                    "y" => {
                                        let y = value
                                            .as_f64()
                                            .ok_or(anyhow!("y should be a number"))?;
                                        y_pos += y;
                                    }
                                    "w" => {
                                        w = value.as_f64().ok_or(anyhow!("w should be a number"))?
                                    }
                                    "h" => {
                                        h = value.as_f64().ok_or(anyhow!("h should be a number"))?
                                    }
                                    "r" => {
                                        r = value.as_f64().ok_or(anyhow!("r should be a number"))?
                                    }
                                    "rx" => {
                                        rx = value
                                            .as_f64()
                                            .ok_or(anyhow!("rx should be a number"))?;
                                        cluster.0 = rx;
                                        x_pos = cluster.0;
                                        y_pos = cluster.1;
                                    }
                                    "ry" => {
                                        ry = value
                                            .as_f64()
                                            .ok_or(anyhow!("ry should be a number"))?;
                                        cluster.1 = ry;
                                        x_pos = cluster.0;
                                        y_pos = cluster.1;
                                    }
                                    "d" => {
                                        decal =
                                            value.as_bool().ok_or(anyhow!("d should be bool"))?
                                    }
                                    "c" => {
                                        let rgb: Srgb<u8> = value
                                            .as_str()
                                            .ok_or(anyhow!("r should be a string"))?
                                            .parse()?;
                                        let (r, g, b) = rgb.into_components();
                                        color = Some((r, b, g));
                                    }
                                    &_ => {
                                        // println!("warning ignored value {:?} = {:?}", key, value)
                                    }
                                }
                            }
                        }
                        Value::String(item) => {
                            // skip decals entirely
                            let labels: Vec<_> = item.split("\n").collect();
                            let (wire, option, encoder) = if labels.len() < 4 {
                                (labels[0], None, false)
                            } else if labels.len() < 10 {
                                (labels[0], Some(labels[3]), false)
                            } else {
                                (labels[0], Some(labels[3]), labels[9].starts_with("e"))
                            };
                            let (xx, yy): (u8, u8) = if let Some((xxx, yyy)) = wire.split_once(',')
                            {
                                if let (Ok(x), Ok(y)) = (xxx.parse(), yyy.parse()) {
                                    (x, y)
                                } else {
                                    (0, 0)
                                }
                            } else {
                                (0, 0)
                            };
                            let layout_options = match option {
                                Some(s) => {
                                    if let Some((l, r)) = s.split_once(',') {
                                        let (l, r) = (l.parse()?, r.parse()?);
                                        if r == 0 {
                                            option_groups.entry(l).or_insert((x_pos, y_pos));
                                        }
                                        Some((l, r))
                                    } else {
                                        None
                                    }
                                }
                                None => None,
                            };
                            let but = if r == 0.0 && rx == 0.0 && ry == 0.0 {
                                /*
                                println!(
                                    "p = {},{}, r = {:?}, rx = {:?}, ry = {:?}, x_pos = {:?}, y_pos = {:?}",
                                    xx, yy, r, rx, ry, x_pos, y_pos,
                                );
                                */
                                let bx = x_pos;
                                let by = y_pos;
                                let bw = w;
                                let bh = h;
                                Button {
                                    x: bx,
                                    y: by,
                                    w: bw,
                                    h: bh,
                                    wire_x: xx,
                                    wire_y: yy,
                                    layout_options,
                                    encoder,
                                    decal,
                                    color,
                                }
                            } else {
                                /*
                                println!(
                                    "p = {},{}, r = {:?}, rx = {:?}, ry = {:?}, x_pos = {:?}, y_pos = {:?}, w = {:?}, h = {:?}",
                                    xx, yy, r, rx, ry, x_pos, y_pos, w, h,
                                );
                                */
                                let theta = -r.to_radians();
                                let theta_sin = theta.sin();
                                let theta_cos = theta.cos();
                                let bx;
                                let by;
                                let x = x_pos - rx;
                                let y = y_pos - ry;
                                if r >= 0.0 {
                                    if r < 45.0 {
                                        bx = x * theta_cos + y * theta_sin + rx;
                                        by = -x * theta_sin + y * theta_cos + ry;
                                    } else {
                                        bx = x * theta_cos + y * theta_sin - h + rx;
                                        by = -x * theta_sin + y * theta_cos + ry;
                                        mem::swap(&mut w, &mut h);
                                    }
                                } else {
                                    // for negative angle rotate right corner
                                    // and shift back -w
                                    // otherwise mirrored part will be
                                    // vertically shifted
                                    if r > -45.0 {
                                        bx = (x + w) * theta_cos + y * theta_sin - w + rx;
                                        by = -(x + w) * theta_sin + y * theta_cos + ry;
                                    } else {
                                        bx = (x + w) * theta_cos + y * theta_sin + rx;
                                        by = -(x + w) * theta_sin + y * theta_cos + ry;
                                        mem::swap(&mut w, &mut h);
                                    }
                                }
                                Button {
                                    x: bx,
                                    y: by,
                                    w,
                                    h,
                                    wire_x: xx,
                                    wire_y: yy,
                                    layout_options: None,
                                    encoder,
                                    decal,
                                    color,
                                }
                                //return Err(MetaParsingError.into());
                            };
                            if matches(&via_options, layout_options) || decal {
                                buttons.push(but);
                            }
                            x_pos += w;
                            w = 1.0;
                            h = 1.0;
                            decal = false;
                            //println!("! {:?} => {:?}", item.as_str().unwrap(), &but);
                        }
                        _ => {
                            return Err(MetaParsingError.into());
                        }
                    }
                }
            }
            None => {
                // sometimes first element is dict we should compensate y_pos increment
                y_pos -= 1.0;
            }
        }
        y_pos += 1.0;
        x_pos = rx;
    }
    // this logic tries to follow via layout_options choices in a following way
    // option_groups contains coordinates of first default (x, 0) button
    // for first button code replaces current choice coordinates (x, current) with coordinates from
    // option_groups then calculates and stores delta between default and current choice
    // for following buttons it applies delta to current coordinates
    let mut deltas = HashMap::new();
    for button in &mut buttons {
        match button.layout_options {
            Some(option) => {
                if matches(&via_options, Some(option)) {
                    match deltas.entry(option.0) {
                        Entry::Vacant(v) => {
                            if let Some((def_x, def_y)) = option_groups.get(&option.0) {
                                //println!("{:?} => {:?}", button, (def_x, def_y));
                                let dx = button.x - *def_x;
                                let dy = button.y - *def_y;
                                button.x = *def_x;
                                button.y = *def_y;
                                v.insert_entry((dx, dy));
                            }
                        }
                        Entry::Occupied(o) => {
                            let (dx, dy) = o.get();
                            //println!("{:?}", (dx, dy));
                            button.x -= dx;
                            button.y -= dy;
                        }
                    }
                }
            }
            None => {
                //do nothing
            }
        }
    }
    Ok(buttons)
}

pub fn render_to_buffer(
    buttons: &Vec<Button>,
    labels: Option<HashMap<(u8, u8), String>>,
) -> Buffer {
    let mut buff = Buffer::new();
    for button in buttons {
        if !button.decal {
            let scale = 4.0;
            let b = button.scale(scale);
            let lu = (b.x.round() as usize, b.y.round() as usize);
            let ru = ((b.x + b.w - 1.0).round() as usize, b.y.round() as usize);
            let lb = (b.x.round() as usize, (b.y + b.h - 1.0).round() as usize);
            let rb = (
                (b.x + b.w - 1.0).round() as usize,
                (b.y + b.h - 1.0).round() as usize,
            );
            if !b.encoder {
                buff.put(lu.0, lu.1, '╔', &b.color);
                for x in (lu.0 + 1)..ru.0 {
                    buff.put(x, lu.1, '═', &b.color);
                }
                buff.put(ru.0, ru.1, '╗', &b.color);
                for y in (lu.1 + 1)..lb.1 {
                    buff.put(lu.0, y, '║', &b.color);
                }
                for y in (ru.1 + 1)..rb.1 {
                    buff.put(ru.0, y, '║', &b.color);
                }
                buff.put(lb.0, lb.1, '╚', &b.color);
                for x in (lb.0 + 1)..rb.0 {
                    buff.put(x, lb.1, '═', &b.color);
                }
                buff.put(rb.0, rb.1, '╝', &b.color);
            } else {
                buff.put(lu.0, lu.1, '╭', &b.color);
                for x in (lu.0 + 1)..ru.0 {
                    buff.put(x, lu.1, '─', &b.color);
                }
                buff.put(ru.0, ru.1, '╮', &b.color);
                for y in (lu.1 + 1)..lb.1 {
                    buff.put(lu.0, y, '│', &b.color);
                }
                for y in (ru.1 + 1)..rb.1 {
                    buff.put(ru.0, y, '│', &b.color);
                }
                buff.put(lb.0, lb.1, '╰', &b.color);
                for x in (lb.0 + 1)..rb.0 {
                    buff.put(x, lb.1, '─', &b.color);
                }
                buff.put(rb.0, rb.1, '╯', &b.color);
            }
            for x in (lu.0 + 1)..ru.0 {
                for y in (lu.1 + 1)..lb.1 {
                    buff.put(x, y, ' ', &b.color);
                }
            }

            let label_x_shift = if b.w < 3.0 { 1 } else { 0 };
            let label_y_shift = if b.h < 3.0 { 1 } else { 0 };

            match labels {
                Some(ref labels) => {
                    if b.encoder {
                        let label = format!(
                            "{}{}",
                            b.wire_x,
                            match b.wire_y {
                                0 => '↺',
                                1 => '↻',
                                _ => 'x',
                            }
                        );
                        for (i, c) in label.chars().enumerate() {
                            buff.put(
                                lu.0 + 1 - label_x_shift + i,
                                lu.1 - label_y_shift + 1,
                                c,
                                &b.color,
                            );
                        }
                    } else {
                        match labels.get(&(b.wire_x, b.wire_y)) {
                            Some(label) => {
                                // FIXME comma treatment is too ugly :( but works
                                let mut we_got_comma = false;
                                for (line, chunk) in label.split(',').enumerate() {
                                    if chunk.is_empty() {
                                        if !we_got_comma {
                                            buff.put(
                                                lu.0 + 1 - label_x_shift + line,
                                                lu.1 - label_y_shift + 1,
                                                ',',
                                                &b.color,
                                            );
                                            we_got_comma = true;
                                        }
                                    } else {
                                        for (i, c) in chunk.chars().enumerate() {
                                            buff.put(
                                                lu.0 + 1 - label_x_shift + i,
                                                lu.1 + 1 - label_y_shift + line,
                                                c,
                                                &b.color,
                                            );
                                        }
                                    }
                                }
                            }
                            None => {
                                // No label => empty button
                            }
                        }
                    }
                }
                None => {
                    let xx = format!("{}", b.wire_x);
                    let yy = format!("{}", b.wire_y);
                    for (i, c) in xx.chars().enumerate() {
                        buff.put(
                            lu.0 + 1 - label_x_shift + i,
                            lu.1 - label_y_shift + 1,
                            c,
                            &b.color,
                        );
                    }
                    for (i, c) in yy.chars().enumerate() {
                        buff.put(
                            lu.0 + 1 - label_x_shift + i,
                            lu.1 - label_y_shift + 2,
                            c,
                            &b.color,
                        );
                    }
                }
            }
        }
    }
    buff
}

pub fn render_and_dump(buttons: &Vec<Button>, labels: Option<HashMap<(u8, u8), String>>) {
    render_to_buffer(buttons, labels).dump();
}
