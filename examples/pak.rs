use std::env;
use std::error::Error;
use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;

use quakeworld::pak::Pak;
#[cfg(feature = "trace")]
use quakeworld::trace::Trace;

fn parse_file(filename: String) -> Result<bool, Box<dyn Error>> {
    // read the file into a buffer
    let file = match File::open(&filename) {
        Ok(file) => file,
        Err(err) => return Err(Box::new(err)),
    };

    #[cfg(feature = "trace")]
    let mut trace = Trace::new();
    let pak = Pak::load(
        filename,
        file,
        #[cfg(feature = "trace")]
        Some(&mut trace),
    )?;

    #[cfg(feature = "trace")]
    for t in trace.trace.traces {
        println!(
            "{} {} start({}) stop({})",
            t.field_name, t.field_type, t.index, t.index_stop
        );
        println!(" value: {:?}", t.value);
        for t1 in &t.traces {
            println!(
                "\t{} {} start({}) stop({})",
                t1.field_type, t1.field_name, t1.index, t1.index_stop
            );
        }
    }
    for file in &pak.files {
        //let b = pak.get_data(file)?;
        println!(
            "{} - {} {}",
            String::from_utf8_lossy(&file.name),
            file.offset,
            file.size
        );
        //println!("{}", b.len());
    }
    return Ok(true);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a pak");
        return;
    }
    let filename = &args[1];
    match parse_file(filename.to_string()) {
        Ok(..) => {
            println!("{} parsed.", filename);
        }
        Err(err) => {
            eprintln!("error in file {}: {}", filename, err);
        }
    }
}
