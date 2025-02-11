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

use protocol_macros::DataTypeRead;
use quakeworld::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use serde::Serialize;

use protocol_macros::DataTypeBoundCheckDerive;

use quakeworld::datatypes::common::DataType;

#[test]
pub fn sized_vector() -> Result<(), quakeworld::vfs::VfsError> {
    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );
    let sized_vector =
        match <quakeworld::datatypes::common::TestSizedVectorSizedString as DataTypeRead>::read(
            &mut datatypereader,
        ) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
    assert_eq!(sized_vector.data.len(), 16);
    assert_eq!(sized_vector.data, raw_data);

    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );
    let sized_vector =
        match <quakeworld::datatypes::common::TestSizedVectorSized as DataTypeRead>::read(
            &mut datatypereader,
        ) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
    assert_eq!(sized_vector.data.len(), 8);
    assert_eq!(sized_vector.data, b"deadbeef");

    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );
    datatypereader.set_env("environment_size", 8);
    let sized_vector =
        match <quakeworld::datatypes::common::TestSizedVectorName as DataTypeRead>::read(
            &mut datatypereader,
        ) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
    assert_eq!(sized_vector.data.len(), 8);
    assert_eq!(sized_vector.data, b"deadbeef");

    let mut raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        trace,
    );
    datatypereader.set_env("environment_size", 4);
    let sized_vector =
        match <quakeworld::datatypes::common::TestSizedVectorName as DataTypeRead>::read(
            &mut datatypereader,
        ) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
    assert_eq!(sized_vector.data.len(), 4);
    assert_eq!(sized_vector.data, b"dead");
    Ok(())
}
