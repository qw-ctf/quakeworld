use paste::paste;
use protocol_macros::DataTypeRead;
/// Structs needed to read the Quakeworld data formats
/// based on: https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm
use serde::Serialize;

use crate::datatypes::common::{BoundingBox, DataType, Vector3, Vertex};
use crate::datatypes::reader::{DataTypeRead, DataTypeReader, DataTypeReaderError};
use crate::trace::{trace_annotate, trace_start, trace_stop};

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl")]
pub struct Header {
    pub magic: u32,
    pub version: u32,
    pub scale: Vector3<f32>,
    pub translate: Vector3<f32>,
    pub bounding_radious: f32,
    pub eye_position: Vector3<f32>,
    pub skin_count: u32,
    pub skin_width: u32,
    pub skin_height: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub frame_count: u32,
    pub sync_types: u32,
    pub flags: u32,
    pub size: f32,
    pub skin_type: u32,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl")]
pub struct Frame {
    pub frame_type: u32,
    pub bounding_box: BoundingBox<Vertex>,
    #[datatyperead(size_from = 16, string)]
    pub name: Vec<u8>,
    #[datatyperead(size_from = "vertex_count")]
    pub vertex: Vec<Vertex>,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl")]
pub struct Skin {
    pub time: f32,
    #[datatyperead(size_from = "skin_size")]
    pub data: Vec<u8>,
}

//
// impl Frame {
//     fn read_special(
//         datatypereader: &mut DataTypeReader,
//         vertex_count: u32,
//     ) -> Result<Self, DataTypeReaderError> {
//         let frame_type = <u32 as DataTypeRead>::read(datatypereader)?;
//         let bounding_box = <BoundingBox<Vertex> as DataTypeRead>::read(datatypereader)?;
//         let name = <MdlFrameName as DataTypeRead>::read(datatypereader)?;
//         let mut vertexes: Vec<Vertex> = Vec::with_capacity(vertex_count as usize);
//         datatypereader.read_exact_generic(&mut vertexes)?;
//         Ok(Frame {
//             frame_type,
//             bounding_box,
//             name,
//             vertexes,
//         })
//     }
//     fn to_datatype(&self) -> DataType {
//         DataType::MDLFRAME(self.clone())
//     }
// }
