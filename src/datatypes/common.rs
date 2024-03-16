use paste::paste;
use quote::quote;
/// Common structs shared between different formats
use serde::Serialize;

use std::ops::Index;

use protocol_macros::DataTypeRead;

use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};

use crate::pak;
#[cfg(feature = "trace")]
use crate::trace::{trace_annotate, trace_start, trace_stop};

use super::bsp;
use super::reader::{MdlFrameName, PakFileName};

/// A trait to get an ascii string
pub trait AsciiString {
    fn ascii_string(&self) -> String;
}

/// A vector or position
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Vector3<T: DataTypeRead + 'static>
where
    T: Clone,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Index<usize> for Vector3<T>
where
    T: DataTypeRead + Clone,
{
    type Output = T;
    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            n => panic!("Invalid Vector3d index: {}", n),
        }
    }
}

/// Bounding box
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead)]
pub struct BoundingBox<T: DataTypeRead + 'static>
where
    T: Clone,
{
    pub min: T,
    pub max: T,
}

#[derive(Debug, Serialize, Clone)]
pub enum BoundingBoxValue<T> {
    BoundingBox(T),
}

#[derive(Serialize, Clone, Debug, Default)]
pub enum DataType {
    #[default]
    None,
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    PAK(crate::pak::Pak),
    PAKHEADER(crate::datatypes::pak::Header),
    PAKFILE(crate::datatypes::pak::File),
    BSPHEADER(bsp::Header),
    DIRECTORYENTRY(DirectoryEntry),
    MDLFRAMENAME(MdlFrameName),
    PAKFILENAME(PakFileName),
    GENERICVECTOR(usize),
}

impl DataType {
    fn to_datatype(self) -> DataType {
        self
    }
}

/// Directory entry: describes the position and size of a chunk of data inside a BSP File
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead)]
pub struct DirectoryEntry {
    /// Offset from the sart of the file
    pub offset: u32,
    /// Size of the chunk
    pub size: u32,
}

impl DataTypeBoundCheck for DirectoryEntry {
    fn check_bounds(&self, datatypereader: &mut DataTypeReader) -> Result<(), DataTypeReaderError> {
        let size = datatypereader.data.len() as u32;
        if self.offset + self.size > size {
            return Err(DataTypeReaderError::BoundCheckError(
                self.offset.into(),
                self.size.into(),
                (self.offset + self.size - size).into(),
                size.into(),
            ));
        }
        Ok(())
    }
}

impl<T: DataTypeBoundCheck> DataTypeBoundCheck for Vec<T> {
    fn check_bounds(&self, datatypereader: &mut DataTypeReader) -> Result<(), DataTypeReaderError> {
        for e in self {
            e.check_bounds(datatypereader)?
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
pub struct TextureCoordinate {
    pub onseam: u32,
    pub s: u32,
    pub t: u32,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead)]
pub struct Triangle {
    pub faces_front: u32,
    pub vertex: Vector3<u32>,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead)]
pub struct Vertex {
    pub v: Vector3<u8>,
    pub normal_index: u8,
}
