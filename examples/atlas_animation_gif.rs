//! [description]
//! takes a bunch of pak files and a map name then generates
//! a gif animation of the atlas texture creation.
//! quite handy for debugging

use std::env;
use std::error::Error;
use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;

use gif::{Encoder, Frame, Repeat};
use std::borrow::Cow;
use std::fs::File;

use quakeworld::bsp::{Bsp, TextureParsed};
use quakeworld::datatypes::common::AsciiString;
use quakeworld::pak::Pak;
use quakeworld::texture::atlas::AtlasTile;
#[cfg(feature = "trace")]
use quakeworld::trace::Trace;
use quakeworld::vfs::{Vfs, VfsFlattenedListEntry, VfsInternalNode, VfsMetaData};

fn create_atlas(paks: Vec<String>, bspname: String) -> Result<bool, Box<dyn Error>> {
    let mut vfs = Vfs::new();
    for pak in paks {
        let pak_data = fs::read(&pak)?;
        let pak = Pak::parse(
            pak,
            pak_data,
            #[cfg(feature = "trace")]
            trace,
        )?;
        let node = VfsInternalNode::new_from_pak(pak, VfsMetaData::default());
        vfs.insert_node(node, "/");
    }

    let mut maps: Vec<(String, Vec<u8>)> = vec![];
    if bspname == "ALLMAPS" {
        let map_names: Vec<String> = vec![];
        let entries = vfs.list("/maps")?;
        for entry in &entries {
            for e in &entry.entries {
                match e {
                    quakeworld::vfs::VfsEntry::File(vfs_entry_file) => {
                        let name = vfs_entry_file.path.last();
                        let name_without_extension = name[0..name.len() - 4].to_string();
                        let name = format!("maps/{}.bsp", name_without_extension);
                        let bsp_data = vfs.read(name, None)?;
                        maps.push((name_without_extension.clone(), bsp_data));
                    }
                    quakeworld::vfs::VfsEntry::Directory(vfs_entry_directory) => {}
                }
            }
        }
    } else {
        let name = format!("maps/{}.bsp", bspname.clone());
        let bsp_data = vfs.read(name, None)?;
        maps.push((bspname.clone(), bsp_data));
    }
    let palette_data = vfs.read("gfx/palette.lmp", None)?;
    // let palette = quakeworld::lmp::Palette::from(palette_data)?;

    for (bsp_name, bsp_data) in maps {
        println!("creating {}_atlas.gif", bsp_name);
        #[cfg(feature = "trace")]
        let mut tr = Trace::new();
        let b = Bsp::parse(
            bsp_data.clone(),
            #[cfg(feature = "trace")]
            Some(&mut tr),
        )?;

        let textures_list = b.textures.clone();
        let statistics = quakeworld::texture::atlas::Statistics::gather(&textures_list, 0);
        let mut atlas = quakeworld::texture::atlas::Atlas::new(512, 512);
        atlas.insert_textures(statistics.textures);
        let tiles_absolut = atlas.tiles.len();

        let limit = b.textures.len();
        let mut count_up = vec![];
        for i in 1..limit {
            count_up.push(i);
        }

        let mut frames: Vec<Vec<_>> = vec![];
        let mut frame_width = 0;
        let mut frame_height = 0;
        for count in count_up {
            let mut textures_list: Vec<TextureParsed> = vec![];
            // textures for statistics
            for n in 0..b.textures.len() as usize {
                textures_list.push(b.textures[n].clone());
            }

            let statistics = quakeworld::texture::atlas::Statistics::gather(&textures_list, 0);
            let mut atlas = quakeworld::texture::atlas::Atlas::new(512, 512);
            atlas.debug_boxes = true;
            let mut new_texture_list = vec![];
            for (c, t) in statistics.textures.iter().enumerate() {
                if c == count {
                    break;
                }
                new_texture_list.push(t.clone());
            }
            atlas.insert_textures(new_texture_list);
            let mut area = 0;
            for (e, tile) in atlas.tiles.iter().enumerate() {
                for t in &tile.textures {}

                for b in &tile.boxes {
                    area += b.size.area();
                }
            }
            if atlas.tiles.len() < tiles_absolut {
                atlas.tiles.push(AtlasTile {
                    size: atlas.tile_size.clone(),
                    boxes: vec![],
                    textures: vec![],
                    full: false,
                });
            }
            let atlas_map = atlas.generate_texture(&b.textures)?;

            frame_height = atlas_map.size.height as u16;
            frame_width = atlas_map.size.width as u16;
            frames.push(atlas_map.data);
        }

        let mut image = File::create(format!("{}_atlas.gif", bsp_name)).unwrap();
        let mut encoder =
            Encoder::new(&mut image, frame_width, frame_height, &palette_data).unwrap();
        encoder.set_repeat(Repeat::Finite(1)).unwrap();
        for frame_data in frames {
            let mut frame = Frame::default();
            frame.width = frame_width;
            frame.height = frame_height;
            frame.delay = 25;
            frame.buffer = Cow::Borrowed(&*frame_data);
            encoder.write_frame(&frame).unwrap();
        }
    }

    Ok(true)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("need to supply at least one pak and a bsp  name. last argument is the map all other are paks");
        return;
    }
    let mut paks: Vec<String> = vec![];
    for i in 1..args.len() - 1 {
        paks.push(args[i].clone());
    }

    let bspname = &args[args.len() - 1];

    match create_atlas(paks, bspname.to_string()) {
        Ok(..) => {
            println!("{} atlas created.", bspname);
        }
        Err(err) => {
            eprintln!("error in file {}: {}", bspname, err);
        }
    }
}
