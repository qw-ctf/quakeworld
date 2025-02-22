use crate::datatypes::bsp::Model;
use crate::datatypes::common::{
    AsciiString, BoundingBox, ClipNode, DirectoryEntry, Edge, Face, Leaf, Node, Vector3,
};
use crate::datatypes::reader::{self, DataTypeSize};
use crate::datatypes::reader::{DataTypeRead, DataTypeReader};
use serde::Serialize;

#[cfg(feature = "trace")]
use crate::trace::Trace;

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
    pub edge_list: Vec<u8>,
    pub clip_nodes: Vec<ClipNode>,
    pub light_maps: Vec<u8>,
    pub leaves: Vec<Leaf>,
}

impl Bsp {
    pub fn parse(
        data: Vec<u8>,
        #[cfg(feature = "trace")] trace: Option<&mut Trace>,
    ) -> Result<Self> {
        let mut dtr = DataTypeReader::new(
            data,
            #[cfg(feature = "trace")]
            trace,
        );
        // read the header
        let bsp_header = <Header as reader::DataTypeRead>::read(&mut dtr)?;

        // parsing all the textures
        let texture_data = dtr.read_data_from_directory_entry(bsp_header.textures)?;
        let mut dtr_texture = DataTypeReader::new(
            texture_data,
            #[cfg(feature = "trace")]
            trace,
        );

        let texture_header =
            <crate::datatypes::common::TextureHeader as DataTypeRead>::read(&mut dtr_texture)?;
        // println!("{:?}", texture_header);

        let mut textures: Vec<TextureParsed> = vec![];
        // reading mip texture info
        for offset in texture_header.offsets {
            // TODO: why is this a thing?
            if offset < 0 {
                continue;
            }
            dtr_texture.set_position(offset as u64);
            let t = <crate::datatypes::common::TextureInfo>::read(&mut dtr_texture)?;
            // println!("{}", t.name.ascii_string());
            let mut mipt_tex: Vec<TextureMip> = vec![];
            let width = t.width;
            let height = t.height;

            // let offset = offset + t.offset1;
            let size = t.width * t.height;
            let d = DirectoryEntry {
                offset: offset as u32,
                size,
            };

            let mut abort = false;
            let mut texture_data = dtr_texture.read_data_from_directory_entry(d)?;
            for (i, off) in vec![
                (1, t.offset1),
                (2, t.offset2),
                (4, t.offset4),
                (8, t.offset8),
            ] {
                let height = t.height / i;
                let width = t.width / i;
                let size = width * height;
                let offset = offset as u32 + off;
                let d = DirectoryEntry { offset, size };
                let data = match dtr_texture.read_data_from_directory_entry(d) {
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
            if abort {
                break;
            }
            textures.push(TextureParsed {
                name: t.name.ascii_string(),
                mip_levels: mipt_tex,
            });
        }

        // read Models
        let models_data = dtr.read_data_from_directory_entry(bsp_header.models)?; //red(&mut datatypereader)?;
        let mut dtr_models = DataTypeReader::new(
            models_data,
            #[cfg(feature = "trace")]
            trace,
        );
        let model_size = Model::datatype_size();
        let model_count = bsp_header.models.size / model_size as u32;
        let mut models: Vec<Model> = Vec::with_capacity(model_count as usize);
        dtr_models.read_exact_generic_v2(&mut models)?;

        // read Vertices
        let vertices_data = dtr.read_data_from_directory_entry(bsp_header.vertices)?;
        let vertex_count = bsp_header.vertices.size / Vector3::<f32>::datatype_size() as u32;
        let mut vertices: Vec<Vector3<f32>> = Vec::with_capacity(vertex_count as usize);
        dtr.read_exact_generic_v2(&mut vertices)?;

        // read Edges
        let edges_data = dtr.read_data_from_directory_entry(bsp_header.edges)?;
        let edge_count = bsp_header.edges.size / Edge::datatype_size() as u32;
        let mut edges: Vec<Edge> = Vec::with_capacity(edge_count as usize);
        dtr.read_exact_generic_v2(&mut edges)?;

        // read Faces
        let faces_data = dtr.read_data_from_directory_entry(bsp_header.faces)?;
        let face_count = bsp_header.faces.size / Face::datatype_size() as u32;
        let mut faces: Vec<Face> = Vec::with_capacity(face_count as usize);
        dtr.read_exact_generic_v2(&mut faces)?;

        // read nodes
        let nodes_data = dtr.read_data_from_directory_entry(bsp_header.nodes)?;
        let node_count = bsp_header.nodes.size / Node::datatype_size() as u32;
        let mut nodes: Vec<Node> = Vec::with_capacity(node_count as usize);
        dtr.read_exact_generic_v2(&mut nodes)?;

        // read leaves
        let leaves_data = dtr.read_data_from_directory_entry(bsp_header.leaves)?;
        let leaves_count = bsp_header.leaves.size / Leaf::datatype_size() as u32;
        let mut leaves: Vec<Leaf> = Vec::with_capacity(leaves_count as usize);
        dtr.read_exact_generic_v2(&mut leaves)?;

        // `read`` lighmaps
        let light_maps = dtr.read_data_from_directory_entry(bsp_header.leaves)?;

        // ClipNodes
        //
        let clipnodes_data = dtr.read_data_from_directory_entry(bsp_header.clipnodes)?;
        let clipnodes_count = bsp_header.clipnodes.size / ClipNode::datatype_size() as u32;
        let mut clip_nodes: Vec<ClipNode> = Vec::with_capacity(clipnodes_count as usize);
        dtr.read_exact_generic_v2(&mut clip_nodes)?;

        let edge_list = dtr.read_data_from_directory_entry(bsp_header.edges_list)?;

        Ok(Bsp {
            header: bsp_header,
            textures,
            models,
            edges,
            edge_list,
            clip_nodes,
            nodes,
            faces,
            vertices,
            light_maps,
            leaves,
        })
    }
}
