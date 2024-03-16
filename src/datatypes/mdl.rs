use paste::paste;
use protocol_macros::DataTypeRead;
/// Structs needed to read the Quakeworld data formats
/// based on: https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm
use serde::Serialize;

use crate::datatypes::common::{BoundingBox, Vector3, Vertex};
use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError, MdlFrameName,
};
use crate::trace::{trace_annotate, trace_start, trace_stop};
use protocol_macros::DataTypeBoundCheckDerive;

use crate::datatypes::common::DirectoryEntry;

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
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
pub struct Frame {
    pub frame_type: u32,
    //pub BoundingBox<Vertex<u8>>,
    pub name: MdlFrameName,
    // pub name: Vec<u8>,
    // pub vertexes: Vec<Vertex>,
}
