use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use paste::paste;
use protocol_macros::DataTypeRead;
use quote::quote;
use serde::Serialize;

use protocol_macros::DataTypeBoundCheckDerive;

use crate::datatypes::common::{DataType, DirectoryEntry};
use crate::datatypes::reader::PakFileName;
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
    pub name: PakFileName,
    pub offset: u32,
    pub size: u32,
}
