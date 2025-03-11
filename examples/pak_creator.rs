use std::env;
use std::error::Error;
use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::io::prelude::*;

use quakeworld::create_pak;
use quakeworld::pak::Pak;
use quakeworld::pak::PakWriter;
#[cfg(feature = "trace")]
use quakeworld::trace::Trace;

use std::iter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a pak");
        return;
    }
    let filename = &args[1];
    // let s = "0123456789".repeat(5);
    // let f1_name = format!("{}{}", s, "01234");
    let s = "aaaaaaaaaa".repeat(5);
    let f1_name = format!("{}{}", s, "aaaaa");
    let f1_data = b"blah";
    let pack = create_pak!((f1_name, f1_data));
    let data = match pack.write_data() {
        Ok(d) => d,
        Err(e) => panic!("{}", e),
    };
    std::fs::write(filename, data).unwrap();
}
