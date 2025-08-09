use std::{collections::HashMap, time::Instant};

use quakeworld::{
    mvd::{Mvd, MvdFrame},
    protocol::message::trace::ReadTrace,
    trace::{self, TraceEntry},
};

use crate::{
    args::{self, TraceviewerArgs},
    read_file, App, DebugValue, TraceReplace, TraceView,
};

pub fn trace_mvd(options: args::TraceCommandMvd) -> Result<TraceView, Box<dyn std::error::Error>> {
    let data = read_file(options.file)?;

    let time_start = Instant::now();
    let mut mvd = Mvd::new(
        data.clone(),
        None,
        // TODO:: fix this to use the quakeworld::trace::TraceOptions
        quakeworld::protocol::message::trace::TraceOptions {
            enabled: true,
            depth_limit: 0,
        },
    )?;
    let mvd_new_time = time_start.elapsed();

    let mut do_trace = true;
    let mut frame = Box::new(MvdFrame::empty());
    let time_start = Instant::now();
    mvd.message.trace.value_track_limit = options.trace_value_depth;
    mvd.message.trace.depth_limit = options.trace_depth_limit;
    let mut error = None;
    while mvd.finished == false {
        if options.frame_start > 0 || options.frame_stop != 0 {
            if mvd.frame >= options.frame_start && mvd.frame <= options.frame_stop {
                do_trace = true;
            } else {
                do_trace = false;
            }
        }
        do_trace = true;
        mvd.message.trace.enabled = do_trace;
        let frame = match mvd.parse_frame() {
            Ok(v) => v,
            Err(e) => {
                error = Some(format!("{:?}", e));
                break;
            }
        };
    }
    let mvd_parse_time = time_start.elapsed();

    let time_start = Instant::now();
    // in case we want to look at the faile/still opened traces change .read to .stack
    let read_traces: Vec<TraceEntry> = mvd
        .message
        .trace
        .read
        .into_iter()
        .map(ReadTrace::from)
        .collect();
    let mut trace_entry = TraceEntry {
        ..Default::default()
    };
    let mvd_read_trace_conversion_time = time_start.elapsed();

    let time_start = Instant::now();
    let stack_traces: Vec<TraceEntry> = mvd
        .message
        .trace
        .stack
        .into_iter()
        .map(ReadTrace::from)
        .collect();
    let mvd_stack_trace_conversion_time = time_start.elapsed();
    let trace_entry = TraceEntry {
        ..Default::default()
    };

    let mut trace_entry_list_read = TraceEntry {
        ..Default::default()
    };
    trace_entry_list_read.traces = read_traces;

    let mut trace_entry_list_stack = TraceEntry {
        ..Default::default()
    };
    trace_entry_list_stack.traces = stack_traces;

    let mut initialization_traces: HashMap<String, DebugValue> = HashMap::new();

    initialization_traces.insert("mvd_new_time".into(), mvd_new_time.into());
    initialization_traces.insert("mvd_parse_time".into(), mvd_parse_time.into());
    initialization_traces.insert(
        "mvd_read_trace_conversion_time".into(),
        mvd_read_trace_conversion_time.into(),
    );
    initialization_traces.insert(
        "mvd_stack_trace_conversion_time".into(),
        mvd_stack_trace_conversion_time.into(),
    );

    let mut read_trace = TraceReplace {
        trace: trace_entry,
        enabled: true,
    };

    Ok(TraceView {
        data: data.clone(),
        read_trace,
        trace_entry_list_read,
        trace_entry_list_stack,
        initialization_traces,
        error,
    })
}
