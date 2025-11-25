use crate::common;
use crate::protocol;
use hidapi::{DeviceInfo, HidApi};
use serde_json::Value;
use std::collections::HashMap;

pub fn run(
    api: &HidApi,
    device: &DeviceInfo,
    qsid: &Option<f64>,
    value: &Option<String>,
    reset: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let device_path = device.path();
    let dev = api.open_path(device_path)?;
    let capabilities = protocol::scan_capabilities(&dev)?;
    if capabilities.vial_version < protocol::VIAL_PROTOCOL_QMK_SETTINGS {
        return Err(common::CommandError("Qmk settings are not supported".to_string()).into());
    }
    if reset {
        if qsid.is_some() || value.is_some() {
            return Err(common::CommandError(
                "Values can be reset only all at once, no qsid nor value should be passed"
                    .to_string(),
            )
            .into());
        }
        protocol::reset_qmk_values(&dev)?;
        return Ok(());
    }
    let qsids = protocol::load_qmk_qsids(&dev)?;
    let settings = protocol::load_qmk_definitions()?;
    match qsid {
        Some(qsid_full) => {
            let qsid_full_str = qsid_full.to_string();
            let tsid: u16;
            let tbit: u8;
            if let Some((l, r)) = qsid_full_str.split_once('.') {
                tsid = l.parse()?;
                tbit = r.parse()?;
            } else {
                tsid = qsid_full_str.parse()?;
                tbit = 0;
            }
            for group in settings["tabs"]
                .as_array()
                .ok_or("tabs should be an array")?
            {
                //let group_name = group["name"].as_str().unwrap();
                for field in group["fields"]
                    .as_array()
                    .ok_or("fields should be an array")?
                {
                    let qsid = field["qsid"].as_u64().ok_or("qsid should be a number")? as u16;
                    let title = field["title"].as_str().ok_or("title should be a string")?;
                    let width: u8 = match &field["width"] {
                        Value::Number(n) => n.as_u64().ok_or("width shoulbe a number")? as u8,
                        _ => 1,
                    };
                    let bool_field =
                        field["type"].as_str().ok_or("type should be string")? == "boolean";
                    let with_bits = !matches!(field["bit"], Value::Null);
                    if qsid == tsid
                        && (!with_bits
                            || (field["bit"].as_u64().ok_or("bit should be number")? as u8) == tbit)
                    {
                        match value {
                            None => {
                                let value = protocol::get_qmk_value(&dev, qsid, width)?;
                                if bool_field {
                                    if with_bits {
                                        println!(
                                            "{}.{}) {} = {}",
                                            qsid,
                                            tbit,
                                            title,
                                            value.get_bool(tbit)
                                        );
                                    } else {
                                        println!("{}) {} = {}", qsid, title, value.get() != 0);
                                    }
                                } else {
                                    println!("{}) {} = {}", qsid, title, value.get());
                                }
                            }
                            Some(v) => {
                                if with_bits {
                                    let mut current_value =
                                        protocol::get_qmk_value(&dev, qsid, width)?.get();
                                    let bw: bool = v.parse()?;
                                    if bw {
                                        current_value |= 1 << tbit;
                                    } else {
                                        current_value ^= 1 << tbit;
                                    }
                                    protocol::set_qmk_value(&dev, qsid, current_value)?;
                                } else if bool_field {
                                    let val: bool = v.parse()?;
                                    let int_val = match val {
                                        true => 1,
                                        false => 0,
                                    };
                                    protocol::set_qmk_value(&dev, qsid, int_val)?;
                                } else {
                                    protocol::set_qmk_value(&dev, qsid, v.parse()?)?;
                                }
                                println!("Option {:?} = {} now", title, v);
                            }
                        }
                    }
                }
            }
        }
        None => {
            let mut values_cache = HashMap::new();

            for group in settings["tabs"]
                .as_array()
                .ok_or("tabs should be an array")?
            {
                let group_name = group["name"].as_str().ok_or("name should be a string")?;
                println!("\n{}:", group_name);
                for field in group["fields"]
                    .as_array()
                    .ok_or("fields should be an array")?
                {
                    let width: u8 = match &field["width"] {
                        Value::Number(n) => n.as_u64().ok_or("width should be a number")? as u8,
                        _ => 1,
                    };
                    let title = field["title"].as_str().ok_or("title should be a string")?;
                    let qsid = field["qsid"].as_u64().ok_or("title should be a number")? as u16;
                    if qsids.contains(&qsid) {
                        let value;
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            values_cache.entry(qsid)
                        {
                            value = protocol::get_qmk_value(&dev, qsid, width)?;
                            e.insert(value);
                        } else {
                            value = *values_cache.get(&qsid).ok_or("cache broken")?;
                        }
                        match field["type"].as_str().ok_or("type should be a string")? {
                            "boolean" => match field["bit"].as_number() {
                                Some(n) => {
                                    let pos = n.as_u64().ok_or("bit should be a number")? as u8;
                                    println!(
                                        "\t{}.{}) {} = {}",
                                        qsid,
                                        pos,
                                        title,
                                        value.get_bool(pos)
                                    );
                                }
                                None => {
                                    println!("\t{}) {} = {}", qsid, title, value.get() != 0);
                                }
                            },
                            _ => {
                                println!("\t{}) {} = {}", qsid, title, value.get());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
