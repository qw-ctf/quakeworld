use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::time::Instant;

use quakeworld::trace::Trace;
use quakeworld::trace::TraceEntry;

use crate::args;
use crate::init_error_hooks;
use crate::init_terminal;
use crate::restore_terminal;
use crate::utils;
use crate::App;
use crate::DebugValue;
use crate::TraceReplace;
use crate::TraceView;

pub fn trace_bsp(options: args::TraceCommandBsp) -> Result<TraceView, Box<dyn Error>> {
    let filename = options.file.clone().into_os_string().into_string().unwrap();
    let data = match options.paks {
        Some(v) => utils::vfs_mount_load(v, filename.clone())?,
        None => crate::read_file(options.file.clone())?,
    };

    let mut trace = Trace::new();
    trace.enabled = true;

    let time_start = Instant::now();
    match quakeworld::bsp::Bsp::parse(data.clone(), Some(trace.clone())) {
        Ok(_) => {}
        Err(e) => return Err(Box::from(format!("{} - bsp parse error", e))),
    };
    let bsp_parse_time = time_start.elapsed();

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

    initialization_traces.insert("bsp parse time".into(), bsp_parse_time.into());

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
