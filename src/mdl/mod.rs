use std::io::Cursor;
use std::io::Read;
use std::io::prelude::*;

use serde::Serialize;
use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MdlError {
    #[error("read error")]
    ReadError,
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("io error {0}")]
    IoError(std::io::Error),
}

impl From<std::io::Error> for MdlError {
    fn from(err: std::io::Error) -> MdlError {
        MdlError::IoError(err)
    }
}

static HEADER_MAGIC: u32 = 1330660425;

#[derive(Serialize, Debug, Default, Clone)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub fn read(mut reader: impl Read) -> Result<Vector, MdlError> {
        Ok(Vector{
        x: reader.read_f32::<LittleEndian>()?,
        y: reader.read_f32::<LittleEndian>()?,
        z: reader.read_f32::<LittleEndian>()?,
        })
    }
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Skin {
    pub time: f32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct TextureCoordinates {
    pub onseam: u32,
    pub s: u32,
    pub t: u32,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Triangle {
    pub faces_front: u32,
    pub vertex: [u32; 3],
}

impl Triangle {
    pub fn read(mut reader: impl Read) -> Result<Triangle, MdlError> {
        Ok(Triangle{
        faces_front: reader.read_u32::<LittleEndian>()?,
        vertex: [reader.read_u32::<LittleEndian>()?, reader.read_u32::<LittleEndian>()?, reader.read_u32::<LittleEndian>()?],
        })
    }
}


#[derive(Serialize, Debug, Default, Clone)]
pub struct Vertex {
    pub v: [u8; 3],
    pub normal_index: u8,
}

impl Vertex {
    pub fn read(mut reader: impl Read) -> Result<Vertex, MdlError> {
        Ok(Vertex{
        v: [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?],
        normal_index: reader.read_u8()?,
        })
    }
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Frame {
    pub frame_type: u32,
    pub bbox_min: Vertex,
    pub bbox_max: Vertex,
    pub name: Vec<u8>,
    pub vertexes: Vec<Vertex>,
}

impl Frame {
    pub fn read(mut reader: impl Read, vertex_count: u32) -> Result<Frame, MdlError> {
        let mut name = vec![0; 16];
        let frame_type = reader.read_u32::<LittleEndian>()?;
        let bbox_min = Vertex::read(&mut reader)?;
        let bbox_max = Vertex::read(&mut reader)?;
        reader.read_exact(&mut name)?;
        let mut vertexes: Vec<Vertex> = Vec::new();
        for _ in 0..vertex_count {
            vertexes.push(Vertex::read(&mut reader)?);
        }
        Ok(Frame{
            frame_type,
            bbox_min,
            bbox_max,
            name,
            vertexes
        })
    }
}


impl TextureCoordinates {
    pub fn read(mut reader: impl Read) -> Result<TextureCoordinates, MdlError> {
        Ok(TextureCoordinates{
        onseam: reader.read_u32::<LittleEndian>()?,
        s: reader.read_u32::<LittleEndian>()?,
        t: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Mdl {
    pub header_magic: u32,
    pub version: u32,
    pub scale: Vector,
    pub translate: Vector,
    pub bounding_radious: f32,
    pub eye_position: Vector,
    pub skin_count: u32,
    pub skin_width: u32,
    pub skin_height: u32,
    pub verxtex_count: u32,
    pub triangle_count: u32,
    pub frame_count: u32,
    pub sync_types: u32,
    pub flags: u32,
    pub size: f32,
    pub skin_type: u32,
    pub skins: Vec<Skin>,
    pub texture_coordinates: Vec<TextureCoordinates>,
    pub triangles: Vec<Triangle>,
    pub frame_type: u32,
    pub frames: Vec<Frame>,
}

impl Mdl {
    pub fn load(mut reader: impl Read) -> Result<Mdl, MdlError> {
        let mut buf: Vec<u8> = Vec::new();
        let mut model: Mdl = Mdl::default();
        reader.read_to_end(&mut buf)?;
        let mut cursor = Cursor::new(buf);
        model.header_magic = cursor.read_u32::<LittleEndian>()?;
        if model.header_magic != HEADER_MAGIC {
            return Err(MdlError::ParseError(format!("header magic number mismatch: expected({}), got({})", HEADER_MAGIC, model.header_magic)));
        }
        model.version = cursor.read_u32::<LittleEndian>()?;

        model.scale = Vector::read(&mut cursor)?;
        model.translate = Vector::read(&mut cursor)?;

        model.bounding_radious = cursor.read_f32::<LittleEndian>()?;
        model.eye_position = Vector::read(&mut cursor)?;
        model.skin_count = cursor.read_u32::<LittleEndian>()?;
        model.skin_width = cursor.read_u32::<LittleEndian>()?;
        model.skin_height = cursor.read_u32::<LittleEndian>()?;

        model.verxtex_count = cursor.read_u32::<LittleEndian>()?;
        model.triangle_count = cursor.read_u32::<LittleEndian>()?;
        model.frame_count = cursor.read_u32::<LittleEndian>()?;

        model.sync_types = cursor.read_u32::<LittleEndian>()?;
        model.flags = cursor.read_u32::<LittleEndian>()?;
        model.size = cursor.read_f32::<LittleEndian>()?;
        model.skin_type = cursor.read_u32::<LittleEndian>()?;
        if model.skin_type == 1 {
            let skin_count = cursor.read_u32::<LittleEndian>()?;
            _ = skin_count;
            panic!("implement me!");
        } else if model.skin_type == 0 {
            let mut buf: Vec<u8> = vec![0; (model.skin_width * model.skin_height) as usize];
            cursor.read_exact(&mut buf)?;
            model.skins.push(Skin{
                time: 0.0,
                data: buf,
            });
        }
        for _ in 0..model.verxtex_count {
            model.texture_coordinates.push(TextureCoordinates::read(&mut cursor)?);
        }

        for _ in 0..model.triangle_count {
            model.triangles.push(Triangle::read(&mut cursor)?);
        }

        for _ in 0..model.frame_count{
            model.frames.push(Frame::read(&mut cursor, model.verxtex_count)?);
        }

        Ok(model)
    }
}



