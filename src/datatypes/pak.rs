use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeSize, Error, Result,
};
use protocol_macros::DataTypeRead;
use serde::Serialize;

use protocol_macros::DataTypeBoundCheckDerive;

use super::common::{DataType, DirectoryEntry};
use crate::trace::{trace_start, trace_stop};

/// PAK related structs
/// PAK Header
type Version = u32;
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, DataTypeBoundCheckDerive)]
#[datatyperead(prefix = "pak", internal)]
pub struct Header {
    /// Pak version
    pub version: Version,
    /// List of files
    #[check_bounds]
    pub directory_offset: DirectoryEntry,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead, DataTypeBoundCheckDerive)]
#[datatyperead(prefix = "pak", internal)]
pub struct File {
    #[datatyperead(size_from = 56, string)]
    pub name: Vec<u8>,
    pub offset: u32,
    pub size: u32,
}

impl File {
    pub fn name_as_string(&self) -> String {
        // @FIXME:  handle this unwrap and all the other crap
        let s = String::from_utf8(self.name.clone()).unwrap();
        let s = s.trim_matches(char::from(0));
        s.to_string()
    }
}
