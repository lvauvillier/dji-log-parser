use binrw::binread;
use serde::Serialize;

use crate::decoder::XorDecoder;

#[binread]
#[derive(Debug)]
#[br(little)]
pub(crate) enum Auxiliary {
    #[br(magic = 0u8)]
    Info(
        #[br(temp)] u16,
        #[br(pad_size_to = self_0, map_stream = |reader| XorDecoder::new(reader, 0))] AuxiliaryInfo,
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
    pub version_data: u8,
    #[br(temp)]
    info_length: u16,
    #[br(count = info_length)]
    pub info_data: Vec<u8>,
    #[br(temp)]
    signature_length: u16,
    #[br(count = signature_length)]
    pub signature_data: Vec<u8>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub(crate) struct AuxiliaryVersion {
    pub version: u16,
    #[br(map = |x: u8| Department::from(x))]
    pub department: Department,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum Department {
    SDK,
    DJIGO,
    DJIFly,
    AgriculturalMachinery,
    Terra,
    DJIGlasses,
    DJIPilot,
    GSPro,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for Department {
    fn from(num: u8) -> Self {
        match num {
            1 => Department::SDK,
            2 => Department::DJIGO,
            3 => Department::DJIFly,
            4 => Department::AgriculturalMachinery,
            5 => Department::Terra,
            6 => Department::DJIGlasses,
            7 => Department::DJIPilot,
            8 => Department::GSPro,
            _ => Department::Unknown(num),
        }
    }
}

impl From<Department> for u8 {
    fn from(department: Department) -> Self {
        match department {
            Department::SDK => 1,
            Department::DJIGO => 2,
            Department::DJIFly => 3,
            Department::AgriculturalMachinery => 4,
            Department::Terra => 5,
            Department::DJIGlasses => 6,
            Department::DJIPilot => 7,
            Department::GSPro => 8,
            Department::Unknown(num) => num,
        }
    }
}
