use crate::{
    args, init_error_hooks, init_terminal, read_file, restore_terminal, App, DebugValue,
    TraceReplace, TraceView,
};
use quakeworld::trace::{Trace, TraceEntry};
use std::{collections::HashMap, error::Error, rc::Rc, time::Instant};

pub fn trace_pak(options: args::TraceCommandPak) -> Result<TraceView, Box<dyn Error>> {
    let data = read_file(options.file)?;

    let mut trace = Trace::new();
    trace.enabled = true;

    let time_start = Instant::now();
    match quakeworld::pak::Pak::parse("dontcare", data.clone(), Some(trace.clone())) {
        Ok(_) => {}
        Err(e) => return Err(Box::from(format!("{} - pak parse error", e))),
    };
    let pak_parse_time = time_start.elapsed();

    let traces = match Rc::try_unwrap(trace.trace) {
        Ok(v) => v.into_inner(),
        Err(_) => return Err(Box::from("unwrap error")),
    };

    let mut trace_entry = TraceEntry {
        ..Default::default()
    };

    let mut trace_entry_list_read = TraceEntry {
        ..Default::default()
    };
    trace_entry_list_read.traces = traces.traces.clone();

    let mut trace_entry_list_stack = TraceEntry {
        ..Default::default()
    };
    trace_entry_list_stack.traces = traces.stack.clone();

    let mut initialization_traces: HashMap<String, DebugValue> = HashMap::new();

    initialization_traces.insert("pak_parse_time".into(), pak_parse_time.into());

    let mut read_trace = TraceReplace {
        trace: traces.clone(),
        enabled: true,
    };

    Ok(TraceView {
        data: data.clone(),
        read_trace,
        trace_entry_list_read,
        trace_entry_list_stack,
        initialization_traces,
    })
}
