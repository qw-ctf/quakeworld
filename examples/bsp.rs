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

fn parse_file(filename: String, bspname: String) -> Result<bool, Box<dyn Error>> {
    let map_name = bspname.clone();
    let bspname = format!("maps/{}.bsp", bspname);
    // read the file into a buffer
    let data = fs::read(&filename)?;
    #[cfg(feature = "trace")]
    let mut trace = Trace::new();
    let pak = Pak::parse(
        filename.clone(),
        data,
        #[cfg(feature = "trace")]
        Some(&mut trace),
    )?;

    let palette = pak
        .files
        .iter()
        .find(|&item| item.name.ascii_string() == "gfx/palette.lmp")
        .or_else(|| {
            println!("\"{}\" not found in \"{}\".", "palette.lmp", filename);
            None
        });
    if !palette.is_some() {
        return Ok(false);
    }
    let palette = pak.get_data(palette.unwrap())?;
    let palette = quakeworld::lmp::Palette::from(palette)?;

    let f = pak
        .files
        .iter()
        .find(|&item| item.name.ascii_string() == bspname)
        .or_else(|| {
            println!("\"{}\" not found in \"{}\".", bspname, filename);
            None
        });
    if !f.is_some() {
        return Ok(false);
    }
    let d = pak.get_data(f.unwrap())?;
    // println!("{}", d.len());
    #[cfg(feature = "trace")]
    let mut tr = Trace::new();
    let b = Bsp::parse(
        d.clone(),
        #[cfg(feature = "trace")]
        Some(&mut tr),
    )?;

    let textures_list = b.textures.clone();
    let statistics = quakeworld::texture::atlas::Statistics::gather(&textures_list, 0);
    let mut atlas = quakeworld::texture::atlas::Atlas::new(512, 512);
    atlas.insert_textures(statistics.textures);
    // println!("atlas tiles: {}", atlas.tiles.len());
    // panic!();
    let tiles_absolut = atlas.tiles.len();

    let limit = 81;
    let mut count_up = vec![];
    for i in 1..82 {
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
        for (e, tile) in atlas.tiles.iter().enumerate() {
            println!("tile {}:", e);
            // println!("\t b: {:?}", tile.boxes);
            for t in &tile.textures {
                println!(
                    "\t t: {} {} - {}",
                    t.position, t.size, b.textures[t.index].name,
                );
            }

            let mut area = 0;
            for b in &tile.boxes {
                println!("\t b: {} {}", b.position, b.size);
                area += b.size.area();
            }
            println!(
                "\t {} - {} = {}",
                area,
                tile.size.area(),
                area as i64 - tile.size.area() as i64
            );
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

        println!(
            "{} - {} {} - {} - {}",
            atlas_map.data.len(),
            atlas.size.width,
            atlas.size.height,
            atlas.size.area(),
            atlas.tiles.len(),
        );
        let png_data = quakeworld::texture::png::from_palette_data(
            &palette,
            &atlas_map.data,
            atlas.size.width,
            atlas.size.height,
        )?;
        println!("png data size: {}", png_data.len());
        println!("atlas tiles: {}", atlas.tiles.len());

        std::fs::write(format!("{}_atlas_{:0>4}.png", map_name, count), png_data);
    }

    Ok(true)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("need to supply a pak and a bsp name");
        return;
    }
    let filename = &args[1];
    let bspname = &args[2];
    match parse_file(filename.to_string(), bspname.to_string()) {
        Ok(..) => {
            println!("{} parsed.", filename);
        }
        Err(err) => {
            eprintln!("error in file {}: {}", filename, err);
        }
    }
}
