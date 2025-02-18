use crate::datatypes::reader::{DataTypeRead, DataTypeReader};

#[cfg(feature = "trace")]
use crate::trace::Trace;

pub type Bsp = crate::datatypes::common::Bsp;
pub type Header = crate::datatypes::bsp::Header;

mod error;
pub use error::{Error, Result};

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
        let bsp_header = <Header as DataTypeRead>::read(&mut dtr)?;

        let texture_data = dtr.read_data_from_directory_entry(bsp_header.textures)?; //red(&mut datatypereader)?;
                                                                                     //
        let mut dtr_texture = DataTypeReader::new(
            texture_data,
            #[cfg(feature = "trace")]
            trace,
        );

        let texture_header =
            <crate::datatypes::common::TextureHeader as DataTypeRead>::read(&mut dtr_texture)?;
        println!("{:?}", texture_header);

        for offset in texture_header.offsets {
            dtr_texture.set_position(offset as u64);
            let t = <crate::datatypes::common::TextureInfo>::read(&mut dtr_texture)?;
            println!("texture: {:?}", t);
        }

        // println!("something: {:?}", bsp.textures);
        // let texture_data = <crate::datatypes::common::TextureHeader

        // println!("texture_header: {:?}", texture_header);
        // let texture_header_offset = datatypereader_texture.position();
        // let mut texture_infos: Vec<crate::datatypes::common::TextureInfo> = vec![];
        // for pos in texture_header.offsets {
        //     datatypereader_texture.set_position(pos as u64);
        //     let t = <crate::datatypes::common::TextureInfo>::read(&mut datatypereader_texture)?;
        //     texture_infos.push(t);
        // }
        //
        // let mut textures: Vec<crate::datatypes::common::Texture> = vec![];
        // for texture_info in texture_infos {
        //     let mut t = crate::datatypes::common::Texture::default();
        //     t.name = String::from_utf8(texture_info.name).unwrap();
        //     t.width = texture_info.width;
        //     t.height = texture_info.height;
        //     let offset = texture_header_offset as u32 + texture_info.offset1;
        //     let size = t.width * t.height;
        //     let d = DirectoryEntry { offset, size };
        //     t.data = datatypereader_texture.read_data_from_directory_entry(d)?;
        //     for (i, off) in vec![
        //         (2, texture_info.offset2),
        //         (4, texture_info.offset4),
        //         (8, texture_info.offset8),
        //     ] {
        //         let size = size / i;
        //         let height = t.height / i;
        //         let width = t.width / i;
        //         let offset = texture_header_offset as u32 + off;
        //         let d = DirectoryEntry { offset, size };
        //         let data = datatypereader_texture.read_data_from_directory_entry(d)?;
        //         t.mips.push(crate::datatypes::common::MipTexture {
        //             width,
        //             height,
        //             data,
        //         });
        //     }
        //     textures.push(t);
        // }
        // println!("{:?}", textures);
        // // println!("{:?}", header);
        // // header.check_bounds(&mut datatypereader)?;

        Ok(Bsp::default())
    }
}
