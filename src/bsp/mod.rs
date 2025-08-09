use crate::datatypes::bsp::Model;
use crate::datatypes::common::{
    AsciiString, BoundingBox, ClipNode, DirectoryEntry, Edge, Face, Leaf, Node, Plane,
    TextureFaceInfo, TextureInfo, Vector3,
};
use crate::datatypes::reader::{self, DataTypeSize};
use crate::datatypes::reader::{DataType, DataTypeRead, DataTypeReader};
use serde::Serialize;

#[cfg(feature = "trace")]
use crate::trace::{Trace, TraceOptional, TraceValue};

use paste::paste;

pub type Header = crate::datatypes::bsp::Header;

mod error;
pub use error::{Error, Result};

#[derive(Serialize, Clone, Debug, Default)]
pub struct TextureMip {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct TextureParsed {
    pub name: String,
    pub mip_levels: Vec<TextureMip>,
}

#[derive(Debug)]
pub struct Bsp {
    pub header: Header,
    pub textures: Vec<TextureParsed>,
    pub texture_infos: Vec<TextureFaceInfo>,
    pub models: Vec<Model>,
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
    pub faces: Vec<Face>,
    pub vertices: Vec<Vector3<f32>>,
    pub edges_list: Vec<i32>,
    pub clip_nodes: Vec<ClipNode>,
    pub light_maps: Vec<u8>,
    pub leaves: Vec<Leaf>,
    pub planes: Vec<Plane>,
}

macro_rules! read_directory_entry {
    ($datatypereader: ident, $directory_entry: expr, $type: ty, $trace_name: literal) => {{
        $datatypereader.set_position($directory_entry.offset as u64);
        trace_start!($datatypereader, $trace_name);
        paste! {
        let size = $type::datatype_size();
        }
        let count = $directory_entry.size / size as u32;
        let mut de_data: Vec<$type> = Vec::with_capacity(count as usize);
        $datatypereader.read_exact_generic_v2(&mut de_data)?;
        trace_stop!($datatypereader);
        de_data
    }};
}

impl Bsp {
    pub fn parse(data: Vec<u8>, #[cfg(feature = "trace")] trace: Option<Trace>) -> Result<Self> {
        let mut dtr = DataTypeReader::new(
            data,
            #[cfg(feature = "trace")]
            trace.clone(),
        );
        // read the header
        let bsp_header = <Header as reader::DataTypeRead>::read(&mut dtr)?;

        // parsing all the textures
        dtr.set_position(bsp_header.textures.offset as u64);
        trace_start!(dtr, "textures");
        let texture_header =
            <crate::datatypes::common::TextureHeader as DataTypeRead>::read(&mut dtr)?;

        let mut textures: Vec<TextureParsed> = vec![];
        // reading mip texture info
        for (count, offset) in texture_header.offsets.into_iter().enumerate() {
            if offset < 0 {
                continue;
            }
            let offset_current = offset as u64 + bsp_header.textures.offset as u64;
            dtr.set_position(offset_current);
            trace_start!(dtr, format!("texture {}", count));
            let t = <crate::datatypes::common::TextureInfo>::read(&mut dtr)?;
            let mut mipt_tex: Vec<TextureMip> = vec![];
            let width = t.width;
            let height = t.height;

            // let offset = offset + t.offset1;
            let size = t.width * t.height;
            let d = DirectoryEntry {
                offset: offset_current as u32,
                size,
            };

            let mut abort = false;
            let mut texture_data = dtr.read_data_from_directory_entry(d)?;
            for (i, off) in vec![
                (1, t.offset1),
                (2, t.offset2),
                (4, t.offset4),
                (8, t.offset8),
            ] {
                let height = t.height / i;
                let width = t.width / i;
                let size = width * height;
                let offset = offset_current as u32 + off;
                let d = DirectoryEntry { offset, size };
                let data = match dtr.read_data_from_directory_entry(d) {
                    Ok(d) => d,
                    Err(_) => {
                        abort = true;
                        break;
                    }
                };
                mipt_tex.push(TextureMip {
                    width,
                    height,
                    data,
                });
            }
            trace_stop!(dtr);
            if abort {
                break;
            }
            textures.push(TextureParsed {
                name: t.name.ascii_string(),
                mip_levels: mipt_tex,
            });
        }
        trace_stop!(dtr);

        let texture_infos = read_directory_entry!(
            dtr,
            bsp_header.texture_info,
            TextureFaceInfo,
            "face texture info"
        );

        let models = read_directory_entry!(dtr, bsp_header.models, Model, "model");

        let vertices = read_directory_entry!(dtr, bsp_header.vertices, Vector3::<f32>, "vertices");

        let edges = read_directory_entry!(dtr, bsp_header.edges, Edge, "edges");

        let faces = read_directory_entry!(dtr, bsp_header.faces, Face, "faces");

        let nodes = read_directory_entry!(dtr, bsp_header.nodes, Node, "nodes");

        let leaves = read_directory_entry!(dtr, bsp_header.leaves, Leaf, "leaves");

        // `read` lighmaps
        trace_start!(dtr, "lightmaps");
        trace_annotate!(dtr, "data");
        let light_maps = dtr.read_data_from_directory_entry(bsp_header.leaves)?;
        trace_stop!(dtr);

        let clip_nodes = read_directory_entry!(dtr, bsp_header.clipnodes, ClipNode, "clipnodes");

        let edges_list = read_directory_entry!(dtr, bsp_header.edges_list, i32, "idgelist");

        let planes = read_directory_entry!(dtr, bsp_header.planes, Plane, "planes");

        Ok(Bsp {
            header: bsp_header,
            textures,
            models,
            edges,
            edges_list,
            clip_nodes,
            nodes,
            faces,
            vertices,
            light_maps,
            leaves,
            planes,
            texture_infos,
        })
    }
}
