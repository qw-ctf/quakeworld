/// Common structs shared between different formats
use serde::Serialize;

use std::ops::Index;

use protocol_macros::DataTypeRead;

use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError, DataTypeSize,
};

use crate::trace::trace_start;

use super::bsp;
use super::mdl;

/// A trait to get an ascii string
pub trait AsciiString {
    fn ascii_string(&self) -> String;
}

/// A vector or position
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
#[datatyperead(ommit_trait = DataTypeSize)]
pub struct Vector3<T: DataTypeRead + 'static>
where
    T: Clone,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: std::clone::Clone + DataTypeRead> DataTypeSize for Vector3<T> {
    fn datatype_size() -> usize {
        std::mem::size_of::<T>() * 3
    }
}

impl<T: DataTypeSize> DataTypeSize for Vec<T> {
    fn datatype_size() -> usize {
        <T as DataTypeSize>::datatype_size()
    }
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
// impl <T> DataTypeRead for Vector3<T> {
//     fn to_datatype(&self) -> DataType {
//         DataType::VECTOR3
//     }
// }
//
// impl<T> DataTypeSize for Vector3<T> {
//     fn datatype_size(&self) -> usize {
//         std::mem::size_of::<T>() * 3
//     }
// }

impl AsciiString for Vec<u8> {
    fn ascii_string(&self) -> String {
        let conv = String::from_utf8_lossy(self);
        conv.chars().filter(|&c| c != '\0').collect()
    }
}

/// Bounding box
#[derive(Serialize, Clone, Debug, Copy, Default, DataTypeRead)]
// #[datatyperead(ommit_func)]
pub struct BoundingBox<T: DataTypeRead + DataTypeSize + 'static>
where
    T: Clone,
{
    pub min: T,
    pub max: T,
}

// impl DataTypeRead for BoundingBox<Vertex> {
// }

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
    BSP(Bsp),
    DIRECTORYENTRY(DirectoryEntry),
    MDLSKIN(mdl::Skin),
    MDLFRAME(mdl::Frame),
    MDLHEADER(mdl::Header),
    GENERICSTRING(String),
    GENERICVECTOR(usize),
    GENERICVECTORSTRING(usize),
    TRIANGLE(Triangle),
    TEXTURECOORDINATE(TextureCoordinate),
    VERTEX(Vertex),
    QTV(crate::qtv::QtvType),
    VECTOR3GENERIC,
    BOUNDINGBOXGENERIC,
    PLANE(Plane),
    TEXTUREHEADER(TextureHeader),
    TEXTUREINFO(TextureInfo),
    TEXTURE(Texture),
    Throwaway,
}

impl DataType {
    #[allow(unused)]
    fn to_datatype(&self) -> DataType {
        self.clone()
    }
}

/// Directory entry: describes the position and size of a chunk of data inside a BSP File
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct DirectoryEntry {
    /// Offset from the sart of the file
    pub offset: u32,
    /// Size of the chunk
    pub size: u32,
}

impl DirectoryEntry {
    pub fn environment(&self, datatypereader: &mut DataTypeReader, name: impl Into<String>) {
        let name = name.into();
        datatypereader.set_env(format!("{}_size", name), self.size);
        datatypereader.set_env(format!("{}_offset", name), self.offset);
    }
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
    pub onseam: i32,
    pub s: i32,
    pub t: i32,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Triangle {
    pub faces_front: u32,
    pub vertex: Vector3<i32>,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Vertex {
    pub v: Vector3<u8>,
    pub normal_index: u8,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub r#type: i32,
}

#[derive(Serialize, Clone, Debug, DataTypeRead, Default)]
pub struct TextureHeader {
    #[datatyperead(environment = "texture_header_count")]
    pub count: i32, // texture count
    #[datatyperead(size_from = "texture_header_count")]
    pub offsets: Vec<i32>,
}

#[derive(Serialize, Clone, Debug, DataTypeRead, Default)]
// #[datatyperead()]
pub struct TextureInfo {
    #[datatyperead(size_from = 16, string)]
    pub name: Vec<u8>,
    pub width: u32,   // width of picture, must be a multiple of ,
    pub height: u32,  // height of picture, must be a multiple of 8
    pub offset1: u32, // offset to u_char Pix[width   * height]
    pub offset2: u32, // offset to u_char Pix[width/2 * height/2]
    pub offset4: u32, // offset to u_char Pix[width/4 * height/4]
    pub offset8: u32, // offset to u_char Pix[width/8 * height/8]
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct MipTexture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Texture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub mips: Vec<MipTexture>,
}

#[derive(Serialize, Clone, Debug, DataTypeRead, Default)]
pub struct Bsp {
    pub header: bsp::Header,
    #[datatyperead(size_offset_from, size=modulo_self_environment)]
    pub planes: Vec<Plane>,
    #[datatyperead(offset_from)]
    pub textures: TextureHeader,
    // #[datatyperead(size_from_directory_entry)]
    // pub textures: Vec<Texture>,
}

// #[derive(Serialize, Clone, Debug, DataTypeRead)]
// pub struct SizedVector {
//     pub capacity: usize,
//     pub data: Vec<u8>,
// }
