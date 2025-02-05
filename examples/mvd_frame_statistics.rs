use quakeworld::mvd::Mvd;
use quakeworld::protocol::types::ServerMessage;
use quakeworld::state::State;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

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
        *buffer,
        #[cfg(feature = "ascii_strings")]
        None,
        #[cfg(feature = "trace")]
        quakeworld::protocol::message::trace::TraceOptions {
            enabled: true,
            depth_limit: 0,
        },
    )?;

    let mut state = State::new();
    let mut frame_average = 0;
    let mut frame_max = 0;
    let mut last_time = -1.0f64;
    let mut frame_count = 0;
    let mut frame_count_start = 0;
    let mut frame_start = 0;
    let mut frame_accum = 0;
    while mvd.finished == false {
        //let frame = mvd.parse_frame()?;
        frame_start = mvd.message.position;
        let frame = match mvd.parse_frame() {
            Ok(v) => v,
            Err(e) => {
                // #[cfg(feature = "trace")]
                // print_message_trace(&mvd.message, false, 0, 2, false)?;
                println!(
                    "max: {} average: {} - frame: {}",
                    frame_max, frame_average, mvd.frame
                );
                return Err(Box::new(e));
            }
        };

        if mvd.frame > 1000 {
            let frame_size = mvd.message.position - frame_start;

            frame_accum += frame_size;

            frame_count += 1;
            if frame.time != last_time {
                if frame_accum > frame_max {
                    frame_max = frame_accum;
                    println!(
                        "max: {} average: {} - frame: {} - frame_start: {}",
                        frame_max, frame_average, mvd.frame, frame_count_start
                    );
                }
                frame_average += frame_accum;
                frame_average = frame_average / 2;
                last_time = frame.time;
                frame_count = 0;
                frame_accum = 0;
                frame_count_start = mvd.frame;
            } else {
            }
        }

        // frame count and demo time
        //println!("--- frame {}:{} ---", frame.frame, frame.time);
        // if you need to keep the last state
        // let old_state = state.clone();
        state.apply_messages_mvd(&frame.messages, frame.last);
        // get the players when intermission is reached
        for message in frame.messages {
            match message {
                ServerMessage::Intermission(_) => {
                    println!("{:#?}", state.players);
                }
                _ => {}
            }
        }
        #[cfg(feature = "trace")]
        if false {
            // if you want to print a trace of each read frame
            print_message_trace(&mvd.message, false, 0, 2, false)?;
        }
        if mvd.finished {
            println!(
                "max: {} average: {} - frame: {}",
                frame_max, frame_average, mvd.frame
            );
        }
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
