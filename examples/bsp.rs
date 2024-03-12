use std::env;
use std::error::Error;
use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;

use quakeworld::bsp::Bsp;
use quakeworld::pak::Pak;
use quakeworld::trace::Trace;

fn parse_file(filename: String, bspname: String) -> Result<bool, Box<dyn Error>> {
    // read the file into a buffer
    let file = match File::open(&filename) {
        Ok(file) => file,
        Err(err) => return Err(Box::new(err)),
    };
    let pak = Pak::parse(filename.clone(), file)?;
    let f = pak
        .files
        .iter()
        .find(|&item| item.name == bspname.as_bytes())
        .or_else(|| {
            println!("\"{}\" not found in \"{}\".", bspname, filename);
            None
        });
    if !f.is_some() {
        return Ok(false);
    }
    let d = pak.get_data(f.unwrap())?;
    println!("{}", d.len());
    let mut tr = Trace::new();
    let b = Bsp::parse(d, &mut tr)?;
    println!("{:?}", b);
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
