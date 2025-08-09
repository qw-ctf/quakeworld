use quakeworld::mvd::Mvd;
#[cfg(feature = "trace")]
use quakeworld::protocol::message::trace::TraceOptions;
use quakeworld::protocol::types::ServerMessage;
use quakeworld::state::State;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};
use time::Time;

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
        TraceOptions::default(),
    )?;

    let mut print_models = true;
    let mut state = State::new();
    let start_time = Instant::now();
    while mvd.finished == false {
        // println!(
        //     "--------------- new frame ({}) ------------------",
        //     mvd.time
        // );
        //let frame = mvd.parse_frame()?;
        let frame = match mvd.parse_frame() {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "trace")]
                print_message_trace(&mvd.message, false, 0, 2, false)?;
                return Err(Box::new(e));
            }
        };

        // frame count and demo time
        //println!("--- frame {}:{} ---", frame.frame, frame.time);
        // if you need to keep the last state
        // let old_state = state.clone();
        state.apply_messages_mvd(&frame.messages, &frame.last);

        // for (index, player) in &state.players {
        //     if player.spectator || player.name.string.len() == 0 {
        //         continue;
        //     }
        // }

        // let mut entities_with_no_model = 0;
        // for (index, ent) in state.entities.iter() {
        //     if ent.model == 0 {
        //         entities_with_no_model += 1;
        //     }
        // }
        // if state.models.len() > 1 && print_models {
        //     for (i, m) in state.models.iter().enumerate() {
        //         println!("{}: {}", i, m.string);
        //     }
        //     print_models = false;
        // }
        // println!(
        //     "{}: entities with no model: {}/{}",
        //     mvd.time,
        //     entities_with_no_model,
        //     state.entities.len()
        // );

        // for (index, ent) in state.entities.iter() {
        //     println!("ent model: {}", ent.model);
        // }
        // get the players when intermission is reached
        // for message in frame.messages {
        //     match message {
        //         ServerMessage::Intermission(_) => {
        //             // println!("{:#?}", state.players);
        //         }
        //         _ => {}
        //     }
        // }
        #[cfg(feature = "trace")]
        if false {
            // if you want to print a trace of each read frame
            print_message_trace(&mvd.message, false, 0, 2, false)?;
        }
    }
    let duration = Instant::now() - start_time;
    println!("{:?}", duration);
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
