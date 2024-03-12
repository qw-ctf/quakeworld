use paste::paste;
use protocol_macros::DataTypeRead;
/// Structs needed to read the Quakeworld data formats
/// based on: https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm
use serde::Serialize;

use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use crate::trace::{trace_start, trace_stop};
use protocol_macros::DataTypeBoundCheckDerive;

use super::common::DirectoryEntry;

/// BSP related structs
/// BSP Header
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, DataTypeBoundCheckDerive)]
pub struct BspHeader {
    /// Model version
    pub version: u32,
    /// List of entities
    #[check_bounds]
    pub entities: DirectoryEntry,
    /// Map planes
    #[check_bounds]
    pub planes: DirectoryEntry,
    /// Textures
    #[check_bounds]
    pub textures: DirectoryEntry,
    /// Vertices
    #[check_bounds]
    pub vertices: DirectoryEntry,
    /// Leaves visibility list
    #[check_bounds]
    pub visibility: DirectoryEntry,
    /// Bsp nodes
    #[check_bounds]
    pub nodes: DirectoryEntry,
    /// Texture info for faces
    #[check_bounds]
    pub tecture_info: DirectoryEntry,
    /// Faces for each surface
    #[check_bounds]
    pub faces: DirectoryEntry,
    /// Lightmaps
    #[check_bounds]
    pub lightmaps: DirectoryEntry,
    /// Clipnodes for models
    #[check_bounds]
    pub clipnodes: DirectoryEntry,
    /// BSP leaves
    #[check_bounds]
    pub leaves: DirectoryEntry,
    /// List of faces
    #[check_bounds]
    pub faces_list: DirectoryEntry,
    /// Edges
    #[check_bounds]
    pub edges: DirectoryEntry,
    /// List of edges
    #[check_bounds]
    pub edges_list: DirectoryEntry,
    /// List of models
    #[check_bounds]
    pub models: DirectoryEntry,
}
