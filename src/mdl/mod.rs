//use crate::mdl::mdl::Frame;
use crate::trace::trace_annotate;
#[cfg(feature = "trace")]
use crate::trace::{Trace, TraceOptional};

use crate::datatypes::common::{DataType, TextureCoordinate, Triangle};
use crate::datatypes::mdl;
use crate::datatypes::reader;
// use crate::datatypes::reader::Error;

use serde::Serialize;

mod error;
pub use error::{Error, Result};

static HEADER_MAGIC: u32 = 1330660425;

#[derive(Serialize, Debug, Default, Clone)]
pub struct Mdl {
    pub header: mdl::Header,
    pub skin: Vec<mdl::SkinType>,
    pub texture_coordinate: Vec<TextureCoordinate>,
    pub triangle: Vec<Triangle>,
    pub frame: Vec<mdl::Frame>,
}

impl Mdl {
    pub fn parse(data: Vec<u8>, #[cfg(feature = "trace")] trace: Option<Trace>) -> Result<Mdl> {
        let mut datatypereader = reader::DataTypeReader::new(
            data,
            #[cfg(feature = "trace")]
            trace,
        );

        let header = <mdl::Header as reader::DataTypeRead>::read(&mut datatypereader)?;

        if header.magic != HEADER_MAGIC {
            return Err(Error::Parse(format!(
                "header magic number mismatch: expected({}), got({})",
                HEADER_MAGIC, header.magic
            )));
        }

        trace_start!(datatypereader, "skins");
        let mut skin: Vec<mdl::SkinType> = vec![];
        for skin_id in 0..header.skin_count {
            trace_start!(datatypereader, format!("skin {}", skin_id));
            trace_annotate!(datatypereader, "type");
            let skin_type = <u32 as reader::DataTypeRead>::read(&mut datatypereader)?;
            if skin_type != 0 {
                let mut skins: Vec<mdl::Skin> = vec![];
                trace_annotate!(datatypereader, "skin_count");
                let skin_count = <u32 as reader::DataTypeRead>::read(&mut datatypereader)?;

                let mut time: Vec<f32> = Vec::with_capacity(skin_count as usize);
                trace_annotate!(datatypereader, "times");
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
                    skins.push(mdl::Skin { time, data });
                }
                skin.push(mdl::SkinType::Group(skins))
            } else if skin_type == 0 {
                // for count in 0..header.skin_count {
                let mut buf: Vec<u8> =
                    Vec::with_capacity((header.skin_width * header.skin_height) as usize);
                trace_annotate!(datatypereader, "skin");
                datatypereader.read_exact(&mut buf)?;

                skin.push(mdl::SkinType::Single(
                    (mdl::Skin {
                        time: 0.0,
                        data: buf,
                    }),
                ));
                // }
            }
            trace_stop!(datatypereader);
        }
        trace_stop!(datatypereader);

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
