use crate::datatypes::bsp::Model;
use crate::datatypes::common::{
    AsciiString, BoundingBox, ClipNode, DirectoryEntry, Edge, Face, Leaf, Node, Vector3,
};
use crate::datatypes::reader::{self, DataTypeSize};
use crate::datatypes::reader::{DataType, DataTypeRead, DataTypeReader};
use serde::Serialize;

#[cfg(feature = "trace")]
use crate::trace::{Trace, TraceOptional, TraceValue};

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

pub struct Bsp {
    pub header: Header,
    pub textures: Vec<TextureParsed>,
    pub models: Vec<Model>,
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
    pub faces: Vec<Face>,
    pub vertices: Vec<Vector3<f32>>,
    pub edges_list: Vec<i16>,
    pub clip_nodes: Vec<ClipNode>,
    pub light_maps: Vec<u8>,
    pub leaves: Vec<Leaf>,
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

        // read Models
        dtr.set_position(bsp_header.models.offset as u64);
        trace_start!(dtr, "models");
        let model_size = Model::datatype_size();
        let model_count = bsp_header.models.size / model_size as u32;
        let mut models: Vec<Model> = Vec::with_capacity(model_count as usize);
        dtr.read_exact_generic_v2(&mut models)?;
        trace_stop!(dtr);

        // read Vertices
        dtr.set_position(bsp_header.vertices.offset as u64);
        trace_start!(dtr, "vertices");
        let vertex_count = bsp_header.vertices.size / Vector3::<f32>::datatype_size() as u32;
        let mut vertices: Vec<Vector3<f32>> = Vec::with_capacity(vertex_count as usize);
        dtr.read_exact_generic_v2(&mut vertices)?;
        trace_stop!(dtr);

        // read Edges
        dtr.set_position(bsp_header.edges.offset as u64);
        trace_start!(dtr, "edges");
        let edge_count = bsp_header.edges.size / Edge::datatype_size() as u32;
        let mut edges: Vec<Edge> = Vec::with_capacity(edge_count as usize);
        dtr.read_exact_generic_v2(&mut edges)?;
        trace_stop!(dtr);

        // read Faces
        dtr.set_position(bsp_header.faces.offset as u64);
        trace_start!(dtr, "faces");
        let faces_data = dtr.read_data_from_directory_entry(bsp_header.faces)?;
        let face_count = bsp_header.faces.size / Face::datatype_size() as u32;
        let mut faces: Vec<Face> = Vec::with_capacity(face_count as usize);
        dtr.read_exact_generic_v2(&mut faces)?;
        trace_stop!(dtr);

        // read nodes
        dtr.set_position(bsp_header.nodes.offset as u64);
        trace_start!(dtr, "nodes");
        let node_count = bsp_header.nodes.size / Node::datatype_size() as u32;
        let mut nodes: Vec<Node> = Vec::with_capacity(node_count as usize);
        dtr.read_exact_generic_v2(&mut nodes)?;
        trace_stop!(dtr);

        // read leaves
        dtr.set_position(bsp_header.leaves.offset as u64);
        trace_start!(dtr, "leaves");
        let leaves_count = bsp_header.leaves.size / Leaf::datatype_size() as u32;
        let mut leaves: Vec<Leaf> = Vec::with_capacity(leaves_count as usize);
        dtr.read_exact_generic_v2(&mut leaves)?;
        trace_stop!(dtr);

        // `read` lighmaps
        trace_start!(dtr, "lightmaps");
        trace_annotate!(dtr, "data");
        let light_maps = dtr.read_data_from_directory_entry(bsp_header.leaves)?;
        trace_stop!(dtr);

        // ClipNodes
        dtr.set_position(bsp_header.clipnodes.offset as u64);
        trace_start!(dtr, "clipnodes");
        let clipnodes_count = bsp_header.clipnodes.size / ClipNode::datatype_size() as u32;
        let mut clip_nodes: Vec<ClipNode> = Vec::with_capacity(clipnodes_count as usize);
        dtr.read_exact_generic_v2(&mut clip_nodes)?;
        trace_stop!(dtr);

        dtr.set_position(bsp_header.edges_list.offset as u64);
        trace_start!(dtr, "edges_list");
        let edges_list_count = bsp_header.edges_list.size / i16::datatype_size() as u32;
        let mut edges_list: Vec<i16> = Vec::with_capacity(edges_list_count as usize);
        dtr.read_exact_generic_v2(&mut edges_list)?;
        trace_stop!(dtr);

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
        })
    }
}
