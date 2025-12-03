use crate::protocol::{CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_ENCODER, CMD_VIAL_SET_ENCODER, send_recv};
use hidapi::HidDevice;

pub fn load_encoder(
    device: &HidDevice,
    layer: u8,
    index: u8,
) -> Result<(u16, u16), Box<dyn std::error::Error>> {
    match send_recv(
        device,
        &[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_ENCODER, layer, index],
    ) {
        Ok(data) => Ok((
            ((data[0] as u16) << 8) + (data[1] as u16),
            ((data[2] as u16) << 8) + (data[3] as u16),
        )),
        Err(e) => Err(e),
    }
}

pub fn set_encoder(
    device: &HidDevice,
    layer: u8,
    index: u8,
    direction: u8,
    value: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match send_recv(
        device,
        &[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_SET_ENCODER,
            layer,
            index,
            direction,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
