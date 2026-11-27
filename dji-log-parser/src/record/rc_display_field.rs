use binrw::binread;
use num_enum::FromPrimitive;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct RCDisplayField {
    #[br(temp)]
    _unknown: [u8; 3],
    #[br(temp)]
    _bitpack1: u8,
    /// Indicates if the record button is pressed
    #[br(calc(_bitpack1 & 0x01 != 0))]
    pub is_record_pressed: bool,
    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(RCModeSwitch::from(_bitpack2 & 0x03)))]
    pub mode_switch: RCModeSwitch,
    #[br(temp)]
    _unknown2: [u8; 2],
    /// right stick - horizontal
    pub aileron: u16,
    /// right stick - vertical
    pub elevator: u16,
    /// left stick - vertical
    pub throttle: u16,
    /// left stick - horizontal
    pub rudder: u16,
    /// left dial, which tilts the gimbal
    pub gimbal: u16,
    /// right dial, from 364 to 1684, 1024 is the center
    pub right_dial: u16,
}

/// Position of the flight mode switch of the remote controller
#[derive(FromPrimitive, Serialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum RCModeSwitch {
    /// S, no flight in sport mode to confirm it
    Sport = 0,
    /// N
    Normal = 1,
    /// T on enterprise drones, C (cine) on consumer drones
    Tripod = 2,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::BinRead;

    #[test]
    fn mode_switch_and_dials() {
        let mut data = [0u8; 19];
        data[3] = 0x01;
        data[4] = 0x02;
        for (index, value) in [1024u16, 1024, 1024, 1024, 364, 1684]
            .into_iter()
            .enumerate()
        {
            data[7 + index * 2..9 + index * 2].copy_from_slice(&value.to_le_bytes());
        }

        let rc = RCDisplayField::read_le(&mut Cursor::new(data)).unwrap();

        assert!(rc.is_record_pressed);
        assert_eq!(rc.mode_switch, RCModeSwitch::Tripod);
        assert_eq!((rc.gimbal, rc.right_dial), (364, 1684));
    }
}
