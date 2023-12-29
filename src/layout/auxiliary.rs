use binrw::binread;
use binrw::io::NoSeek;

use crate::decoder::MagicDecoder;

#[binread]
#[derive(Debug)]
#[br(little)]
pub enum Auxiliary {
    #[br(magic = 0u8)]
    Info(
        #[br(temp)] u16,
        #[br(pad_size_to = self_0, map_stream = |reader| NoSeek::new(MagicDecoder::new(reader, 0)))]
         AuxiliaryInfo,
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
pub struct AuxiliaryInfo {
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
pub struct AuxiliaryVersion {
    pub version: u16,
    pub department: u8,
}
