use std::fs::File;

use quakeworld::datatypes::common::AsciiString;
use quakeworld::pak::Pak;
use quakeworld::utils::perf::Perf;
use quakeworld::vfs::internal_node::VfsFlattenedListEntry;
use quakeworld::vfs::{
    internal_node::VfsInternalNode, path::VfsPath, Vfs, VfsEntryDirectory, VfsEntryFile, VfsNode,
    VfsQueryDirectory, VfsQueryFile,
};

use paste::paste;

use quakeworld::trace_start;

use protocol_macros::DataTypeRead;
use quakeworld::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError, DataTypeSize,
};
use serde::Serialize;

use protocol_macros::DataTypeBoundCheckDerive;

use quakeworld::datatypes::common::DataType;

#[derive(Serialize, Clone, Debug, DataTypeRead)]
#[datatyperead(datatype = Throwaway)]
struct SizedVectorName {
    #[datatyperead(size_from = "environment_size")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Clone, Debug, DataTypeRead)]
#[datatyperead(datatype = Throwaway)]
pub struct SizedVectorSizedString {
    #[datatyperead(size_from = 16, string)]
    pub data: Vec<u8>,
}

#[derive(Serialize, Clone, Debug, DataTypeRead)]
#[datatyperead(datatype = Throwaway)]
pub struct SizedVectorSized {
    #[datatyperead(size_from = 8, string)]
    pub data: Vec<u8>,
}

#[test]
pub fn sized_vector_string_sized() -> Result<(), quakeworld::vfs::Error> {
    // this ill read all 16 bytes but stop reading to the Vec when the first \0 is encountered
    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();
    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );
    let sized_vector = match <SizedVectorSizedString as DataTypeRead>::read(&mut datatypereader) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };
    assert_eq!(datatypereader.cursor.position(), 16);
    assert_eq!(sized_vector.data.len(), 8);
    assert_eq!(sized_vector.data, b"deadbeef");
    Ok(())
}

#[test]
pub fn sized_vector_sized() -> Result<(), quakeworld::vfs::Error> {
    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );
    let sized_vector = match <SizedVectorSized as DataTypeRead>::read(&mut datatypereader) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };
    assert_eq!(sized_vector.data.len(), 8);
    assert_eq!(sized_vector.data, b"deadbeef");
    Ok(())
}

#[test]
pub fn sized_vector_sized_named_from_environment() -> Result<(), quakeworld::vfs::Error> {
    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();
    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );

    // missing environment variable
    let sized_vector = match <SizedVectorName as DataTypeRead>::read(&mut datatypereader) {
        Ok(v) => panic!("we should have errord' here"),
        Err(e) => {
            if let DataTypeReaderError::EnvironmentVariableNotFound(v) = e {
                assert_eq!(v, "environment_size");
            } else {
                panic!("encountered wrong error: {}", e);
            }
        }
    };

    datatypereader.set_env("environment_size", 12);
    let sized_vector = match <SizedVectorName as DataTypeRead>::read(&mut datatypereader) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };
    assert_eq!(sized_vector.data.len(), 12);
    assert_eq!(sized_vector.data, b"deadbeef\0\0\0\0");

    Ok(())
}
