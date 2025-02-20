use std::env;
use std::error::Error;
use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;

use quakeworld::bsp::Bsp;
use quakeworld::datatypes::common::AsciiString;
use quakeworld::pak::Pak;
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

    let atlas_map = quakeworld::texture::atlas::Atlas::from_textures(b.textures);
    let png_data = quakeworld::texture::png::from_palette_data(
        &palette,
        &atlas_map.data,
        atlas_map.width,
        atlas_map.height,
    )?;

    std::fs::write(format!("{}_atlas.png", map_name), png_data);

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
