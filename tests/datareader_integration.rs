//! [description]
//! datatypes::reader tests

#![allow(warnings)]
use std::fs::File;

use quakeworld::datatypes::common::AsciiString;
use quakeworld::pak::Pak;
use quakeworld::utils::perf::Perf;
use quakeworld::vfs::{
    path::VfsPath, Vfs, VfsEntryDirectory, VfsEntryFile, VfsNode, VfsQueryDirectory, VfsQueryFile,
};

use paste::paste;

use quakeworld::{trace_start, trace_stop};

use quakeworld::datatypes::reader::{DataTypeBoundCheck, DataTypeRead, DataTypeReader};
use serde::Serialize;

use protocol_macros::DataTypeRead;

use quakeworld::datatypes::common::DataType;

macro_rules! generate_data {
    ($($data:expr),*) => {
    {
        let mut v: Vec<u8> = vec![];

    $(
        paste!{
        v.append(&mut $data.clone())
        }
    )*
        v
    }
    }
}

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
    let raw_data = b"deadbeef\0\0\0\0\0\0\0\0".to_vec();
    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        None,
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
        None,
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
        None,
    );

    // missing environment variable
    let sized_vector = match <SizedVectorName as DataTypeRead>::read(&mut datatypereader) {
        Ok(v) => panic!("we should have errord' here"),
        Err(e) => {
            if let quakeworld::datatypes::reader::Error::EnvironmentVariableNotFound(v) = e {
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

/// environment tests
#[derive(Serialize, Clone, Debug, DataTypeRead)]
#[datatyperead(datatype = Throwaway)]
pub struct EnvironmentDirectoryEntry {
    #[datatyperead(environment)]
    pub entry: quakeworld::datatypes::common::DirectoryEntry,
    #[datatyperead(size_from = "entry_size")]
    pub data: Vec<u8>,
}

#[test]
pub fn environment_directory_entry_size_sequential(
) -> Result<(), quakeworld::datatypes::reader::Error> {
    let original_data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let raw_data = generate_data!(vec![9, 0, 0, 0, 8, 0, 0, 0], &mut original_data.clone());

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        None,
    );

    let directory_entry =
        match <EnvironmentDirectoryEntry as DataTypeRead>::read(&mut datatypereader) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    assert_eq!(directory_entry.entry.size, 8);
    assert_eq!(directory_entry.entry.offset, 9);
    let size: u64 = datatypereader.get_env_error("entry_size")?.into();
    assert_eq!(size, 8);
    let offset: u64 = datatypereader.get_env_error("entry_offset")?.into();
    assert_eq!(offset, 9);
    assert_eq!(directory_entry.data, vec![0, 1, 2, 3, 4, 5, 6, 7]);

    Ok(())
}

#[derive(Serialize, Clone, Debug, DataTypeRead)]
#[datatyperead(datatype = Throwaway)]
pub struct EnvironmentDirectoryEntryBoth {
    #[datatyperead(environment)]
    pub entry: quakeworld::datatypes::common::DirectoryEntry,
    #[datatyperead(size_offset_from = "entry")]
    pub data: Vec<u8>,
}

#[test]
pub fn environment_directory_entry_size_offset() -> Result<(), quakeworld::datatypes::reader::Error>
{
    let original_data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let size_expected = original_data.len();
    let mut raw_data = generate_data!(vec![16, 0, 0, 0, 8, 0, 0, 0], b"DEADBEEF".to_vec());
    let offset_expected = raw_data.len();
    let raw_data = generate_data!(raw_data, &mut original_data.clone());
    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        None,
    );

    let directory_entry =
        match <EnvironmentDirectoryEntryBoth as DataTypeRead>::read(&mut datatypereader) {
            Ok(v) => v,
            Err(e) => {
                println!("---------->we are here?");
                println!("{:?}", datatypereader.env);
                return Err(e);
            }
        };
    assert_eq!(directory_entry.entry.size, size_expected as u32);
    assert_eq!(directory_entry.entry.offset, offset_expected as u32);
    let size: u64 = datatypereader.get_env_error("entry_size")?.into();
    assert_eq!(size, size_expected as u64);
    let offset: u64 = datatypereader.get_env_error("entry_offset")?.into();
    assert_eq!(offset, offset_expected as u64);
    assert_eq!(directory_entry.data, original_data);
    Ok(())
}

#[derive(Serialize, Clone, Debug, DataTypeRead)]
#[datatyperead(datatype = Throwaway)]
pub struct EnvironmentDirectoryEntryComplex {
    #[datatyperead(environment)]
    pub entry1: quakeworld::datatypes::common::DirectoryEntry,
    #[datatyperead(environment)]
    pub entry2: quakeworld::datatypes::common::DirectoryEntry,

    #[datatyperead(size_offset_from = "entry1")]
    pub data1: Vec<u8>,
    #[datatyperead(size_offset_from = "entry2")]
    pub data2: Vec<u8>,
}

#[test]
pub fn environment_directory_entry_size_offset_complex(
) -> Result<(), quakeworld::datatypes::reader::Error> {
    let sizeof_directory_entry: u8 =
        std::mem::size_of::<quakeworld::datatypes::common::DirectoryEntry>()
            .try_into()
            .unwrap(); //

    let original_data_1: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let size_expected_1 = original_data_1.len() as u8;
    let offset_expected_1 = sizeof_directory_entry * 2 + 8;

    let original_data_2: Vec<u8> = vec![7, 6, 5, 4, 3, 2, 1, 0];
    let size_expected_2 = original_data_2.len() as u8;
    let offset_expected_2 = offset_expected_1 + size_expected_1 + 8;

    let mut raw_data = generate_data!(
        vec![offset_expected_1, 0, 0, 0, size_expected_1, 0, 0, 0],
        vec![offset_expected_2, 0, 0, 0, size_expected_2, 0, 0, 0],
        b"DEADBEEF".to_vec(),
        &mut original_data_1.clone(),
        b"DEADBEEF".to_vec(),
        &mut original_data_2.clone()
    );

    let mut datatypereader = DataTypeReader::new(
        raw_data.clone(),
        #[cfg(feature = "trace")]
        None,
    );

    let directory_entry =
        match <EnvironmentDirectoryEntryComplex as DataTypeRead>::read(&mut datatypereader) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    assert_eq!(directory_entry.entry1.size, size_expected_1 as u32);
    assert_eq!(directory_entry.entry1.offset, offset_expected_1 as u32);
    assert_eq!(directory_entry.data1, original_data_1);
    assert_eq!(directory_entry.entry2.size, size_expected_2 as u32);
    assert_eq!(directory_entry.entry2.offset, offset_expected_2 as u32);
    assert_eq!(directory_entry.data2, original_data_2);
    Ok(())
}
