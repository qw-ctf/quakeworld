use std::env;
use std::error::Error;
use std::fs::{self, FileType};
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;
use std::rc::Rc;

use quakeworld::bsp::{Bsp, TextureParsed};
use quakeworld::datatypes::common::AsciiString;
use quakeworld::pak::Pak;
use quakeworld::texture::atlas::AtlasTile;

use quakeworld::trace::Trace;
use quakeworld::vfs::{Vfs, VfsInternalNode, VfsMetaData};

// fn parse_file(filename: String, bspname: String) -> Result<bool, Box<dyn Error>> {
//     let map_name = bspname.clone();
//     let bspname = format!("maps/{}.bsp", bspname);
//     // read the file into a buffer
//     let data = fs::read(&filename)?;
//
//     let mut trace = Trace::new();
//     let pak = Pak::parse(
//         filename.clone(),
//         data,
//         #[cfg(feature = "trace")]
//         Some(&mut trace),
//     )?;
//
//     let palette = pak
//         .files
//         .iter()
//         .find(|&item| item.name.ascii_string() == "gfx/palette.lmp")
//         .or_else(|| {
//             println!("\"{}\" not found in \"{}\".", "palette.lmp", filename);
//             None
//         });
//     if !palette.is_some() {
//         return Ok(false);
//     }
//     let palette = pak.get_data(palette.unwrap())?;
//     let palette = quakeworld::lmp::Palette::from(palette)?;
//
//     let f = pak
//         .files
//         .iter()
//         .find(|&item| item.name.ascii_string() == bspname)
//         .or_else(|| {
//             println!("\"{}\" not found in \"{}\".", bspname, filename);
//             None
//         });
//     if !f.is_some() {
//         return Ok(false);
//     }
//     let d = pak.get_data(f.unwrap())?;
//     // println!("{}", d.len());
//     #[cfg(feature = "trace")]
//     let mut tr = Trace::new();
//     let b = Bsp::parse(
//         d.clone(),
//         #[cfg(feature = "trace")]
//         Some(&mut tr),
//     )?;
//
//     let textures_list = b.textures.clone();
//     let statistics = quakeworld::texture::atlas::Statistics::gather(&textures_list, 0);
//     let mut atlas = quakeworld::texture::atlas::Atlas::new(512, 512);
//     atlas.insert_textures(statistics.textures);
//     // println!("atlas tiles: {}", atlas.tiles.len());
//     // panic!();
//     let tiles_absolut = atlas.tiles.len();
//
//     let limit = 81;
//     let mut count_up = vec![];
//     for i in 1..82 {
//         count_up.push(i);
//     }
//     for count in count_up {
//         let mut textures_list: Vec<TextureParsed> = vec![];
//         // textures for statistics
//         for n in 0..b.textures.len() as usize {
//             textures_list.push(b.textures[n].clone());
//         }
//
//         let statistics = quakeworld::texture::atlas::Statistics::gather(&textures_list, 0);
//         let mut atlas = quakeworld::texture::atlas::Atlas::new(512, 512);
//         let mut new_texture_list = vec![];
//         for (c, t) in statistics.textures.iter().enumerate() {
//             if c == count {
//                 break;
//             }
//             new_texture_list.push(t.clone());
//         }
//         atlas.insert_textures(new_texture_list);
//         for (e, tile) in atlas.tiles.iter().enumerate() {
//             println!("tile {}:", e);
//             // println!("\t b: {:?}", tile.boxes);
//             for t in &tile.textures {
//                 println!(
//                     "\t t: {} {} - {}",
//                     t.position, t.size, b.textures[t.index].name,
//                 );
//             }
//
//             let mut area = 0;
//             for b in &tile.boxes {
//                 println!("\t b: {} {}", b.position, b.size);
//                 area += b.size.area();
//             }
//             println!(
//                 "\t {} - {} = {}",
//                 area,
//                 tile.size.area(),
//                 area as i64 - tile.size.area() as i64
//             );
//         }
//         if atlas.tiles.len() < tiles_absolut {
//             atlas.tiles.push(AtlasTile {
//                 size: atlas.tile_size.clone(),
//                 boxes: vec![],
//                 textures: vec![],
//                 full: false,
//             });
//         }
//         let atlas_map = atlas.generate_texture(&b.textures)?;
//
//         println!(
//             "{} - {} {} - {} - {}",
//             atlas_map.data.len(),
//             atlas.size.width,
//             atlas.size.height,
//             atlas.size.area(),
//             atlas.tiles.len(),
//         );
//         let png_data = quakeworld::texture::png::from_palette_data(
//             &palette,
//             &atlas_map.data,
//             atlas.size.width,
//             atlas.size.height,
//         )?;
//         println!("png data size: {}", png_data.len());
//         println!("atlas tiles: {}", atlas.tiles.len());
//
//         std::fs::write(format!("{}_atlas_{:0>4}.png", map_name, count), png_data);
//     }
//
//     Ok(true)
// }
#[derive(PartialEq)]
enum QuakeworldFileType {
    Unknown,
    Bsp,
    Pak,
    Mdl,
}

fn classify_file(filename: &String) -> QuakeworldFileType {
    let t = match filename.char_indices().nth_back(2) {
        Some(c) => c.0,
        None => return QuakeworldFileType::Unknown,
    };
    match &filename[t..] {
        "bsp" => return QuakeworldFileType::Bsp,
        "mdl" => return QuakeworldFileType::Mdl,
        "pak" => return QuakeworldFileType::Pak,
        _ => return QuakeworldFileType::Unknown,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("need to supply [a list of paks] and a filename");
        return;
    }
    let filename = args.last().unwrap().to_string();
    let cft = classify_file(&filename);
    if cft == QuakeworldFileType::Unknown {
        println!("can only trace bsp, mdl, and pak");
        return;
    }

    let data: Vec<u8> = if args.len() > 2 {
        let paks = &args[1..args.len() - 1];
        let mut vfs = Vfs::new();
        for pak in paks {
            let pak_data = match fs::read(&pak) {
                Ok(v) => v,
                Err(e) => {
                    println!("{} - couldnt load {}", e, pak);
                    return;
                }
            };
            let pak = match Pak::parse(pak, pak_data, None) {
                Ok(p) => p,
                Err(e) => {
                    println!("{} - couldnt parse {}", e, pak);
                    return;
                }
            };
            let node = VfsInternalNode::new_from_pak(pak, VfsMetaData::default());
            vfs.insert_node(node, "/");
        }

        match vfs.read(filename.clone(), None) {
            Ok(d) => d,
            Err(e) => {
                println!("{} - couldnt find {}", e, filename);
                return;
            }
        }
    } else {
        match fs::read(filename.clone()) {
            Ok(v) => v,
            Err(e) => {
                println!("{} - couldnt load {}", e, filename);
                return;
            }
        }
    };

    let mut trace = quakeworld::trace::Trace::new();
    trace.enabled = true;

    match cft {
        QuakeworldFileType::Unknown => {
            println!("this is weird we shouldnt be here");
            return;
        }
        QuakeworldFileType::Bsp => match quakeworld::bsp::Bsp::parse(data, Some(trace.clone())) {
            Ok(_) => {}
            Err(e) => println!("{} - couldnt parse bsp {}", e, filename),
        },
        QuakeworldFileType::Pak => {
            match quakeworld::pak::Pak::parse("dontcare", data, Some(trace.clone())) {
                Ok(_) => {}
                Err(e) => println!("{} - couldnt parse pak {}", e, filename),
            }
        }
        QuakeworldFileType::Mdl => match quakeworld::mdl::Mdl::parse(data, Some(trace.clone())) {
            Ok(_) => {}
            Err(e) => println!("{} - couldnt parse pak {}", e, filename),
        },
    }
    let trace_done = match Rc::try_unwrap(trace.trace) {
        Ok(v) => v.into_inner(),
        Err(_) => {
            println!("error unwrapping");
            return;
        }
    };
    let json = match serde_json::to_string(&trace_done) {
        Ok(json) => json,
        Err(e) => {
            println!("{} - serialization error", e);
            return;
        }
    };

    let mut file = match std::fs::File::create(format!("{}.json", filename)) {
        Ok(f) => f,
        Err(e) => {
            println!("{} - file error for {}.json", e, filename);
            return;
        }
    };

    match file.write_all(json.as_bytes()) {
        Ok(_) => {
            println!(
                "'{}' trace written to '{}.json' written",
                filename, filename
            );
        }
        Err(e) => {
            println!("{} - error writing '{}.json'", e, filename)
        }
    }
}
