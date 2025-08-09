use protocol_macros::DataTypeRead;
/// Structs needed to read the Quakeworld data formats
/// based on: https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm
use serde::Serialize;

use super::common::{BoundingBox, DataType, Vector3, Vertex};
use super::reader::{DataTypeRead, DataTypeReader, DataTypeSize, Error, Result};
use crate::datatypes::reader;
use crate::trace::{trace_annotate, trace_start, trace_stop};

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl", internal)]
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
}

#[derive(Serialize, Debug, Default, Clone)]
// #[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
// #[datatyperead(prefix = "mdl", internal)]
pub struct Frame {
    pub frame_type: u32,
    pub frame: FrameType,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl", internal)]
pub struct FrameSimple {
    pub bounding_box: BoundingBox<Vertex>,
    #[datatyperead(size_from = 16, string)]
    pub name: Vec<u8>,
    #[datatyperead(size_from = "vertex_count")]
    pub vertex: Vec<Vertex>,
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl", internal)]
pub struct FrameGroup {
    #[datatyperead(environment)]
    pub count: u32,
    pub bounding_box: BoundingBox<Vertex>,
    #[datatyperead(size_from = "count")]
    pub times: Vec<f32>,
    #[datatyperead(size_from = "count")]
    pub frames: Vec<FrameSimple>,
}

#[derive(Serialize, Debug, Default, Clone)]
pub enum FrameType {
    #[default]
    None,
    Single(FrameSimple),
    Group(FrameGroup),
}

impl DataTypeRead for Frame {
    fn read(dtr: &mut DataTypeReader) -> Result<Self> {
        trace_annotate!(dtr, "type");
        let frame_type = <u32 as reader::DataTypeRead>::read(dtr)?;
        let frame = if frame_type == 0 {
            FrameType::Single(<FrameSimple as reader::DataTypeRead>::read(dtr)?)
        } else {
            FrameType::Group(<FrameGroup as reader::DataTypeRead>::read(dtr)?)
        };
        Ok(Self { frame_type, frame })
    }

    fn to_datatype(&self) -> DataType {
        DataType::MDLFRAME(self.clone())
    }

    fn environment(&self, datatypereader: &mut DataTypeReader, name: impl Into<String>) {
        panic!("we do get called?");
        // compile_error!("you need to implement the environment function");
    }
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
#[datatyperead(prefix = "mdl", internal)]
pub struct Skin {
    pub time: f32,
    #[datatyperead(size_from = "skin_size")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Debug, Default, Clone)]
pub enum SkinType {
    #[default]
    None,
    Single(Skin),
    Group(Vec<Skin>),
}
