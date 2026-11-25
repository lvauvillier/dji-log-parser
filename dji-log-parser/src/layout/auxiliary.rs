use binrw::binread;
use serde::Serialize;

use crate::decoder::xor_decoder;
use num_enum::{FromPrimitive, IntoPrimitive};

#[binread]
#[derive(Debug)]
#[br(little)]
pub(crate) enum Auxiliary {
    #[br(magic = 0u8)]
    Info(
        #[br(temp)] u16,
        #[br(pad_size_to = self_0, map_stream = |reader| xor_decoder(reader, 0))] AuxiliaryInfo,
    ),

    #[br(magic = 1u8)]
    Version(
        #[br(temp)] u16,
        #[br(pad_size_to = self_0)] AuxiliaryVersion,
    ),
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub(crate) struct AuxiliaryInfo {
    #[br(temp)]
    _version_data: u8,
    #[br(temp)]
    info_length: u16,
    #[br(count = info_length)]
    pub info_data: Vec<u8>,
    #[br(temp)]
    signature_length: u16,
    #[br(temp, count = signature_length)]
    _signature_data: Vec<u8>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub(crate) struct AuxiliaryVersion {
    pub version: u16,
    #[br(map = |x: u8| Department::from(x))]
    pub department: Department,
}

#[derive(FromPrimitive, IntoPrimitive, Serialize, Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum Department {
    SDK = 1,
    DJIGO = 2,
    DJIFly = 3,
    AgriculturalMachinery = 4,
    Terra = 5,
    DJIGlasses = 6,
    DJIPilot = 7,
    GSPro = 8,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}
