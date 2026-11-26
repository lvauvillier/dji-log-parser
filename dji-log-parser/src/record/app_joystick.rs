use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// Sticks of the app, on aircraft flown from the mobile device.
///
/// Stick positions range from -1 to 1, 0 is the center.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { length: u16 })]
#[br(assert(usize::from(length) <= DATA_LENGTH || crc8(&data) == crc, "invalid app joystick checksum"))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct AppJoystick {
    /// Buttons, 7 channels of 11 bits (the sticks are the first 4) and switches
    #[br(temp)]
    data: [u8; DATA_LENGTH],
    #[br(temp)]
    crc: u8,

    /// right stick - horizontal
    #[br(calc(stick(&data, 0)))]
    pub aileron: f32,
    /// right stick - vertical
    #[br(calc(stick(&data, 1)))]
    pub elevator: f32,
    /// left stick - vertical
    #[br(calc(stick(&data, 2)))]
    pub throttle: f32,
    /// left stick - horizontal
    #[br(calc(stick(&data, 3)))]
    pub rudder: f32,
}

const DATA_LENGTH: usize = 13;

/// Stick position, from a channel of 11 bits after the first byte: from 364 to 1684, 1024 is the
/// center.
fn stick(data: &[u8; DATA_LENGTH], index: u32) -> f32 {
    const CENTER: f32 = 1024.0;
    const RANGE: f32 = 660.0;

    let bits = u64::from_le_bytes(data[1..9].try_into().expect("8 bytes"));
    let channel = (bits >> (11 * index)) & 0x7FF;
    (channel as f32 - CENTER) / RANGE
}

/// CRC-8 of polynomial 0x31 (reflected), with an initial value of 0x77.
fn crc8(data: &[u8]) -> u8 {
    data.iter().fold(0x77, |crc, byte| {
        (0..8).fold(crc ^ byte, |crc, _| {
            if crc & 1 == 1 {
                (crc >> 1) ^ 0x8C
            } else {
                crc >> 1
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::{args, BinRead};

    fn read(data: &[u8]) -> binrw::BinResult<AppJoystick> {
        let length = data.len() as u16;
        AppJoystick::read_args(&mut Cursor::new(data), args! { length })
    }

    #[test]
    fn sticks_are_the_first_channels() {
        let joystick = read(&[
            0x00, 0x22, 0xf4, 0x2f, 0x00, 0x01, 0x08, 0x40, 0x00, 0x02, 0x10, 0x00, 0x06, 0x86,
        ])
        .unwrap();

        // Channels of 1058, 1534, 1024 and 1024
        assert_eq!(joystick.aileron, 34.0 / 660.0);
        assert_eq!(joystick.elevator, 510.0 / 660.0);
        assert_eq!(joystick.throttle, 0.0);
        assert_eq!(joystick.rudder, 0.0);
    }

    #[test]
    fn sticks_range_from_minus_1_to_1() {
        let mut data = [0; DATA_LENGTH];
        // Channels 0 and 1 at 364 and 1684
        let bits: u64 = 364 | (1684 << 11);
        data[1..9].copy_from_slice(&bits.to_le_bytes());
        let mut bytes = data.to_vec();
        bytes.push(crc8(&data));

        let joystick = read(&bytes).unwrap();

        assert_eq!((joystick.aileron, joystick.elevator), (-1.0, 1.0));
    }

    #[test]
    fn checksum_is_checked() {
        let result = read(&[
            0x00, 0x22, 0xf4, 0x2f, 0x00, 0x01, 0x08, 0x40, 0x00, 0x02, 0x10, 0x00, 0x06, 0x87,
        ]);

        assert!(result.is_err());
    }
}
