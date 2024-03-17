use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use paste::paste;
use protocol_macros::DataTypeRead;
use serde::Serialize;

use protocol_macros::DataTypeBoundCheckDerive;

use crate::datatypes::common::{DataType, DirectoryEntry};
use crate::trace::{trace_annotate, trace_start, trace_stop};
/// PAK related structs
/// PAK Header
type Version = u32;
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, DataTypeBoundCheckDerive)]
pub struct Header {
    /// Pak version
    pub version: Version,
    /// List of files
    #[check_bounds]
    pub directory_offset: DirectoryEntry,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead, DataTypeBoundCheckDerive)]
pub struct File {
    #[datatyperead(size = 56, string)]
    pub name: Vec<u8>,
    pub offset: u32,
    pub size: u32,
}
