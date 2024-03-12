use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use paste::paste;
use protocol_macros::DataTypeRead;
use serde::Serialize;

use protocol_macros::DataTypeBoundCheckDerive;

use crate::trace::{trace_start, trace_stop, TraceValue};
/// PAK related structs
/// PAK Header
type Version = u32;
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, DataTypeBoundCheckDerive)]
pub struct PakHeader {
    /// Model version
    pub version: Version,
    /// List of entities
    #[check_bounds]
    pub directory_offset: super::common::DirectoryEntry,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead, DataTypeBoundCheckDerive)]
pub struct File {
    pub name: super::reader::PakFileName,
    pub offset: u32,
    pub size: u32,
}
