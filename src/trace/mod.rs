use crate::datatypes::{common::DataType, reader::DataTypeRead};
use serde::Serialize;
use std::any::Any;

#[derive(Serialize, Clone, Debug, Default)]
pub enum TraceValue {
    #[default]
    None,
}

#[derive(Debug)]
pub struct TraceEntry {
    pub annotation: String,
    pub index: u64,
    pub size: u64,
    pub value: Option<Box<dyn Any>>,
    pub traces: Vec<TraceEntry>,
    stack: Vec<TraceEntry>,
}

#[derive(Debug, Default)]
pub struct Trace {
    pub traces: Vec<TraceEntry>,
    stack: Vec<TraceEntry>,
}

impl Trace {
    pub fn new() -> Self {
        Trace {
            traces: vec![],
            stack: vec![],
        }
    }
    pub fn start(self: &mut Self, index: u64, annotation: impl Into<String>) {
        let annotation = annotation.into();
        let ts = TraceEntry {
            annotation,
            index,
            size: 0,
            traces: vec![],
            stack: vec![],
            value: None,
        };
        self.stack.push(ts);
    }

    pub fn annotate(self: &mut Self, annptation_prepend: impl Into<String>) {
        // pop the most recent trace
    }

    pub fn stop(self: &mut Self, size: u64, value: Option<Box<dyn Any>>) {
        // pop the most recent trace
        if let Some(mut p) = self.stack.pop() {
            p.value = value;
            p.size = size;
            // get the last trace on the stack
            if let Some(l) = self.stack.last_mut() {
                l.size += p.size;
                // put that trace on the last element in the stack if it exists
                l.traces.push(p);
            } else {
                // if not the trace is finished
                self.traces.push(p);
            }
        } else {
            panic!("ok?");
        }
    }
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[cfg(not(feature = "trace"))]
macro_rules! trace_start {}

#[cfg(feature = "trace")]
macro_rules! trace_start {
    ( $dr:ident, $name:expr) => {
        if let Some(trace) = &mut $dr.trace {
            trace.start($dr.cursor.position(), $name);
        }
    };
    ( $dr:ident, $name:expr) => {
        if let Some(trace) = $dr.trace {
            trace.start($dr.cursor.position(), $name);
        }
    };
}
pub(crate) use trace_start;

#[cfg(not(feature = "trace"))]
macro_rules! trace_stop {}

#[cfg(feature = "trace")]
macro_rules! trace_stop {
    ( $dr:ident, $value:expr, $valueType:ident) => {
        paste! {

        if let Some(trace) = &mut $dr.trace {
            trace.stop($dr.cursor.position(), Some(Box::new($value.clone())));
        }
        }
    };
    ($self:expr, $value:expr) => {
        // if $self.trace.enabled && !$self.trace.locked {
        //     $self.read_trace_stop($value.to_tracevalue());
        // }
    };
    ($self:expr) => {
        // if $self.trace.enabled && !$self.trace.locked {
        //     $self.read_trace_stop(TraceValue::None);
        // }
    };
}
pub(crate) use trace_stop;

#[cfg(not(feature = "trace"))]
macro_rules! trace_annotate {}

#[cfg(feature = "trace")]
macro_rules! trace_annotate {
    ( $dr:ident, $name:expr) => {
        if let Some(trace) = &mut $dr.trace {
            trace.annotate($name);
        }
    };
    ( $dr:ident, $name:expr) => {
        if let Some(trace) = $dr.trace {
            trace.annotate($name);
        }
    };
}
pub(crate) use trace_annotate;
