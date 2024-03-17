//use crate::mdl::mdl::Frame;
#[cfg(feature = "trace")]
use crate::trace::Trace;
use crate::trace::{trace_annotate};

use crate::datatypes::common::{TextureCoordinate, Triangle};
use crate::datatypes::mdl;
use crate::datatypes::reader;
use crate::datatypes::reader::DataTypeReaderError;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MdlError {
    #[error("read error")]
    ReadError,
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("io error {0}")]
    IoError(std::io::Error),
    #[error("{0}")]
    DataTypeReaderError(DataTypeReaderError),
}

impl From<DataTypeReaderError> for MdlError {
    fn from(err: DataTypeReaderError) -> MdlError {
        MdlError::DataTypeReaderError(err)
    }
}

impl From<std::io::Error> for MdlError {
    fn from(err: std::io::Error) -> MdlError {
        MdlError::IoError(err)
    }
}

static HEADER_MAGIC: u32 = 1330660425;
//
// #[derive(Serialize, Debug, Default, Clone)]
// pub struct Vector {
//     pub x: f32,
//     pub y: f32,
//     pub z: f32,
// }
//
// impl Vector {
//     pub fn read(mut reader: impl Read) -> Result<Vector, MdlError> {
//         Ok(Vector {
//             x: reader.read_f32::<LittleEndian>()?,
//             y: reader.read_f32::<LittleEndian>()?,
//             z: reader.read_f32::<LittleEndian>()?,
//         })
//     }
// }

// #[derive(Serialize, Debug, Default, Clone)]
// pub struct Frame {
// pub frame_type: u32,
// pub BoundingBox<u8>,
// pub name: Vec<u8>,
// pub vertexes: Vec<Vertex>,
// }

// impl Frame {
//     pub fn read(mut reader: impl Read, vertex_count: u32) -> Result<Frame, MdlError> {
//         let mut name = vec![0; 16];
//         let frame_type = reader.read_u32::<LittleEndian>()?;
//         let bbox_min = Vertex::read(&mut reader)?;
//         let bbox_max = Vertex::read(&mut reader)?;
//         reader.read_exact(&mut name)?;
//         let mut vertexes: Vec<Vertex> = Vec::new();
//         for _ in 0..vertex_count {
//             vertexes.push(Vertex::read(&mut reader)?);
//         }
//         Ok(Frame {
//             frame_type,
//             bbox_min,
//             bbox_max,
//             name,
//             vertexes,
//         })
//     }
// }

#[derive(Serialize, Debug, Default, Clone)]
pub struct Mdl {
    pub header: mdl::Header,
    pub skin: Vec<mdl::Skin>,
    pub texture_coordinate: Vec<TextureCoordinate>,
    pub triangle: Vec<Triangle>,
    pub frame: Vec<mdl::Frame>,
    // pub header_magic: u32,
    // pub version: u32,
    // pub scale: Vector,
    // pub translate: Vector,
    // pub bounding_radious: f32,
    // pub eye_position: Vector,
    // pub skin_count: u32,
    // pub skin_width: u32,
    // pub skin_height: u32,
    // pub verxtex_count: u32,
    // pub triangle_count: u32,
    // pub frame_count: u32,
    // pub sync_types: u32,
    // pub flags: u32,
    // pub size: f32,
    // pub skin_type: u32,
    // pub skins: Vec<Skin>,
    // pub texture_coordinates: Vec<TextureCoordinates>,
    // pub triangles: Vec<Triangle>,
    // pub frame_type: u32,
    // pub frames: Vec<Frame>,
}

impl Mdl {
    pub fn parse(
        data: Vec<u8>,
        #[cfg(feature = "trace")] trace: Option<&mut Trace>,
    ) -> Result<Mdl, MdlError> {
        let mut datatypereader = reader::DataTypeReader::new(
            data,
            #[cfg(feature = "trace")]
            trace,
        );

        let header = <mdl::Header as reader::DataTypeRead>::read(&mut datatypereader)?;

        if header.magic != HEADER_MAGIC {
            return Err(MdlError::ParseError(format!(
                "header magic number mismatch: expected({}), got({})",
                HEADER_MAGIC, header.magic
            )));
        }

        println!("{:?}", header);
        // let mut buf: Vec<u8> = Vec::new();
        // let mut model: Mdl = Mdl::default();
        // reader.read_to_end(&mut buf)?;
        // let mut cursor = Cursor::new(buf);
        // model.header_magic = cursor.read_u32::<LittleEndian>()?;
        // if model.header_magic != HEADER_MAGIC {
        //     return Err(MdlError::ParseError(format!(
        //         "header magic number mismatch: expected({}), got({})",
        //         HEADER_MAGIC, model.header_magic
        //     )));
        // }
        // model.version = cursor.read_u32::<LittleEndian>()?;
        //
        // model.scale = Vector::read(&mut cursor)?;
        // model.translate = Vector::read(&mut cursor)?;
        //
        // model.bounding_radious = cursor.read_f32::<LittleEndian>()?;
        // model.eye_position = Vector::read(&mut cursor)?;
        // model.skin_count = cursor.read_u32::<LittleEndian>()?;
        // model.skin_width = cursor.read_u32::<LittleEndian>()?;
        // model.skin_height = cursor.read_u32::<LittleEndian>()?;
        //
        // model.verxtex_count = cursor.read_u32::<LittleEndian>()?;
        // model.triangle_count = cursor.read_u32::<LittleEndian>()?;
        // model.frame_count = cursor.read_u32::<LittleEndian>()?;
        //
        // model.sync_types = cursor.read_u32::<LittleEndian>()?;
        // model.flags = cursor.read_u32::<LittleEndian>()?;
        // model.size = cursor.read_f32::<LittleEndian>()?;
        // model.skin_type = cursor.read_u32::<LittleEndian>()?;
        let mut skin: Vec<mdl::Skin> = vec![];
        if header.skin_type == 1 {
            trace_annotate!(datatypereader, "skin_count");
            let skin_count = <u32 as reader::DataTypeRead>::read(&mut datatypereader)?;

            let mut time: Vec<f32> = Vec::with_capacity(skin_count as usize);
            trace_annotate!(datatypereader, "skin_times");
            datatypereader.read_exact_generic(&mut time)?;

            let mut skin_data: Vec<Vec<u8>> = vec![];
            for _ in 0..skin_count {
                let mut data: Vec<u8> =
                    Vec::with_capacity((header.skin_width * header.skin_height) as usize);
                trace_annotate!(datatypereader, "skin_data");
                datatypereader.read_exact_generic(&mut data)?;
                skin_data.push(data);
            }
            for (time, data) in time.into_iter().zip(skin_data.into_iter()) {
                skin.push(mdl::Skin { time, data });
            }
        } else if header.skin_type == 0 {
            let mut buf: Vec<u8> = vec![0; (header.skin_width * header.skin_height) as usize];
            trace_annotate!(datatypereader, "skin_data");
            datatypereader.read_exact(&mut buf)?;
            skin.push(mdl::Skin {
                time: 0.0,
                data: buf,
            });
        }

        let mut texture_coordinate: Vec<TextureCoordinate> =
            Vec::with_capacity(header.vertex_count as usize);

        trace_annotate!(datatypereader, "TextureCoordinate");
        datatypereader.read_exact_generic(&mut texture_coordinate)?;

        let mut triangle: Vec<Triangle> = Vec::with_capacity(header.triangle_count as usize);
        trace_annotate!(datatypereader, "Triangle");
        datatypereader.read_exact_generic(&mut triangle)?;

        let mut frame: Vec<mdl::Frame> = Vec::with_capacity(header.frame_count as usize);
        trace_annotate!(datatypereader, "Frame");
        datatypereader.set_env("vertex_count", header.vertex_count as i64);
        datatypereader.read_exact_generic(&mut frame)?;

        Ok(Mdl {
            header,
            skin,
            texture_coordinate,
            triangle,
            frame,
        })
    }
}
