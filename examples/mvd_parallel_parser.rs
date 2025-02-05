use core::time;
use quakeworld::mvd::Mvd;
use quakeworld::protocol::message::trace::TraceOptions;
use quakeworld::protocol::types::ServerMessage;
use quakeworld::state::State;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use std::{env, thread};

#[cfg(feature = "trace")]
use quakeworld::utils::trace::*;

// the most basic implementation of a mvd parser
fn parse_file(filename: String) -> Result<bool, Box<dyn Error>> {
    // read the file into a buffer
    let mut buffer = Box::new(Vec::new());
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => return Err(Box::new(err)),
    };
    match file.read_to_end(&mut buffer) {
        Ok(size) => size,
        Err(err) => return Err(Box::new(err)),
    };
    let mut mvd = Mvd::new(
        *buffer.clone(),
        #[cfg(feature = "ascii_strings")]
        None,
        #[cfg(feature = "trace")]
        TraceOptions {
            enabled: true,
            depth_limit: 0,
        },
    )?;
    let start_time = Instant::now();
    match mvd.parse_mutlithreaded(8) {
        Ok(r) => println!("{:?}", r.len()),
        Err(e) => eprintln!("{}", e),
    };
    println!("parallel duration: {:?}", start_time.elapsed());
    let sleep = time::Duration::new(20, 0);
    thread::sleep(sleep);

    let start_time = Instant::now();
    let mut mvd = Mvd::new(
        *buffer.clone(),
        #[cfg(feature = "ascii_strings")]
        None,
        #[cfg(feature = "trace")]
        TraceOptions {
            enabled: true,
            depth_limit: 0,
        },
    )?;
    while mvd.finished == false {
        match mvd.parse_frame() {
            Ok(_) => {}
            Err(_) => break,
        };
    }
    println!("singlethread duration: {:?}", start_time.elapsed());
    thread::sleep(sleep);

    #[cfg(feature = "trace")]
    if false {
        // if you want to print a trace of each read frame
        print_message_trace(&mvd.message, false, 0, 2, false)?;
    }
    return Ok(true);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a demo");
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
