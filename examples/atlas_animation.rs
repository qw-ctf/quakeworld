use std::env;
use std::error::Error;
use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;

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
            None,
        )?;
        let node = VfsInternalNode::new_from_pak(pak, VfsMetaData::default());
        vfs.insert_node(node, "/");
    }

    let map_name = bspname.clone();
    let bspname = format!("maps/{}.bsp", bspname);

    let bsp_data = vfs.read(bspname, None)?;
    let palette_data = vfs.read("gfx/palette.lmp", None)?;
    let palette = quakeworld::lmp::Palette::from(palette_data)?;

    let b = Bsp::parse(
        bsp_data.clone(),
        #[cfg(feature = "trace")]
        None,
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
    for count in count_up {
        let mut textures_list: Vec<TextureParsed> = vec![];
        // textures for statistics
        for n in 0..b.textures.len() as usize {
            textures_list.push(b.textures[n].clone());
        }

        let statistics = quakeworld::texture::atlas::Statistics::gather(&textures_list, 0);
        let mut atlas = quakeworld::texture::atlas::Atlas::new(512, 512);
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

        let png_data = quakeworld::texture::png::from_palette_data(
            &palette,
            &atlas_map.data,
            atlas_map.size.width,
            atlas_map.size.height,
        )?;

        _ = std::fs::write(format!("{}_atlas_{:0>4}.png", map_name, count), png_data)?;
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
