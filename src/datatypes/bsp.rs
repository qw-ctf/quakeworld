use protocol_macros::DataTypeRead;
/// Structs needed to read the Quakeworld data formats
/// based on: https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm
use serde::Serialize;

use super::{
    common::{BoundingBox, Vector3},
    reader::{DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeSize, Error},
};
use crate::trace::{trace_start, trace_stop};
use protocol_macros::DataTypeBoundCheckDerive;

use crate::datatypes::common::{DataType, DirectoryEntry};

/// BSP related structs
/// BSP Header
///https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, DataTypeBoundCheckDerive, Default)]
#[datatyperead(prefix = "bsp", internal)]
pub struct Header {
    /// Model version
    pub version: u32,
    /// List of entities
    #[check_bounds]
    pub entities: DirectoryEntry,

    /// Map planes
    #[check_bounds]
    #[datatyperead(environment)]
    pub planes: DirectoryEntry,

    /// Textures
    #[check_bounds]
    #[datatyperead(environment)]
    pub textures: DirectoryEntry,

    /// Vertices
    #[check_bounds]
    #[datatyperead(environment)]
    pub vertices: DirectoryEntry,

    /// Leaves visibility list
    #[check_bounds]
    #[datatyperead(environment)]
    pub visibility: DirectoryEntry,

    /// Bsp nodes
    #[check_bounds]
    #[datatyperead(environment)]
    pub nodes: DirectoryEntry,

    /// Texture info for faces
    #[check_bounds]
    #[datatyperead(environment)]
    pub texture_info: DirectoryEntry,

    /// Faces for each surface
    #[check_bounds]
    #[datatyperead(environment)]
    pub faces: DirectoryEntry,

    /// Lightmaps
    /// i guess they are just an array of u8
    #[check_bounds]
    #[datatyperead(environment)]
    pub lightmaps: DirectoryEntry,

    /// Clipnodes for models
    #[check_bounds]
    #[datatyperead(environment)]
    pub clipnodes: DirectoryEntry,

    /// BSP leaves
    #[check_bounds]
    #[datatyperead(environment)]
    pub leaves: DirectoryEntry,
    /// List of faces
    #[check_bounds]
    #[datatyperead(environment)]
    pub faces_list: DirectoryEntry,

    /// Edges
    #[check_bounds]
    #[datatyperead(environment)]
    pub edges: DirectoryEntry,

    /// List of edges
    /// TODO: still not read
    #[check_bounds]
    #[datatyperead(environment)]
    pub edges_list: DirectoryEntry,

    /// List of models
    #[check_bounds]
    #[datatyperead(environment)]
    pub models: DirectoryEntry,
}

#[derive(Serialize, Clone, Debug, Default, DataTypeRead)]
#[datatyperead(prefix = "bsp", internal)]
pub struct Model {
    pub bounding_box: BoundingBox<Vector3<f32>>,
    pub origin: Vector3<f32>,
    pub node_id0: i32,
    pub node_id1: i32,
    pub node_id2: i32,
    pub node_id3: i32,
    pub leafs_count: i32,
    pub face_iindex: i32,
    pub face_count: i32,
}
