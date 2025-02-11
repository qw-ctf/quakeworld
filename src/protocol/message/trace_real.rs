use std::collections::HashMap;

use crate::protocol::message::DeltaUserCommand;
use crate::protocol::message::StringByte;
use crate::protocol::message::StringVector;
use crate::trace;
use paste::paste;
use serde::Serialize;
use strum_macros::Display;

use crate::mvd::MvdFrame;
use crate::protocol::message::{Message, Packet, ServerMessage};
use crate::trace::{TraceBase, TraceEntry};

#[derive(Serialize, Clone, Debug, Default)]
pub struct TraceOptions {
    pub enabled: bool,
    pub depth_limit: u32,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ReadTrace {
    pub start: usize,
    pub stop: usize,
    pub readahead: bool,
    pub aborted: bool,
    pub function: String,
    pub annotation: Option<String>,
    pub read: Vec<ReadTrace>,
    pub value: TraceValue,
    pub info: HashMap<String, String>,
}

/*
#[derive(Serialize, Clone, Debug, Default)]
pub enum TraceValue {
    #[default] None,
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    VecU8(Vec<u8>),
    ServerMessage(ServerMessage),
    Packet(Packet),
    StringByte(StringByte),
    StringVector(StringVector),
}
*/

#[derive(Serialize, Clone, Debug)]
pub struct MessageTrace {
    pub annotation: Option<String>,
    pub stack: Vec<ReadTrace>,
    pub read: Vec<ReadTrace>,
    pub enabled: bool,
    pub value_track_limit: i32,
    pub depth_limit: i32,
    pub depth: i32,
    pub locked: bool,
}
impl Default for MessageTrace {
    fn default() -> Self {
        MessageTrace {
            annotation: None,
            stack: vec![],
            read: vec![],
            enabled: false,
            value_track_limit: -1,
            depth_limit: -1,
            depth: 0,
            locked: false,
        }
    }
}

impl TraceBase for MessageTrace {
    fn get_trace(self) -> Vec<TraceEntry> {
        let traces: Vec<TraceEntry> = self.read.into_iter().collect();
        traces
    }
}

impl From<ReadTrace> for TraceEntry {
    fn from(a: ReadTrace) -> Self {
        let field_name = a.annotation.unwrap_or("".to_owned());
        let traces: Vec<TraceEntry> = a.read.into_iter().collect();
        let index_stop = if a.stop > 0 { (a.stop - 1) as u64 } else { 0 };
        TraceEntry {
            field_type: a.function,
            field_name,
            index: a.start as u64,
            index_stop,
            value: trace::TraceValue::Message(a.value),
            traces,
            stack: vec![],
            info: a.info,
        }
    }
}

impl FromIterator<ReadTrace> for Vec<TraceEntry> {
    fn from_iter<T: IntoIterator<Item = ReadTrace>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let mut vec = Vec::new();
        for item in iterator {
            vec.push(TraceEntry::from(item));
        }
        vec
    }
}

impl MessageTrace {
    pub fn clear(&mut self) {
        self.stack.clear();
        self.read.clear();
        self.annotation = None;
    }
}

pub(crate) trait ToTraceValue {
    fn to_tracevalue(&self) -> TraceValue;
}

pub trait PrintType {
    fn print_type(&self) -> String;
}

impl Message {
    pub fn read_trace_annotate(&mut self, annotation: impl Into<String>) {
        let annotation = annotation.into();
        if !self.trace.enabled {
            return;
        }
        self.trace.annotation = Some(annotation);
    }

    pub fn read_trace_start(&mut self, function: impl Into<String>, readahead: bool) {
        if !self.trace.enabled {
            return;
        }
        self.trace.depth += 1;
        if self.trace.depth_limit != -1 && self.trace.depth_limit <= self.trace.depth {
            return;
        }
        let function = function.into();
        let mut annotation = None;
        if self.trace.annotation.is_some() {
            annotation = self.trace.annotation.clone();
            self.trace.annotation = None;
        }
        let res = ReadTrace {
            function,
            start: self.position,
            readahead,
            stop: self.position,
            read: vec![],
            value: TraceValue::None,
            annotation,
            aborted: false,
            info: HashMap::new(),
        };
        self.trace.stack.push(res)
    }

    pub fn read_trace_abort(&mut self) {
        if !self.trace.enabled {
            return;
        }
        if let Some(mut trace) = self.trace.stack.pop() {
            trace.aborted = true;
            trace.stop = self.position;

            let len = self.trace.stack.len();
            if len > 0 {
                self.trace.stack[len - 1].read.push(trace);
            } else {
                self.trace.read.push(trace);
            }
        }
    }

    pub fn read_trace_stop(&mut self, value: TraceValue, function: impl Into<String>) {
        let function = function.into();
        if !self.trace.enabled {
            return;
        }
        if self.trace.depth_limit != -1 && self.trace.depth_limit <= self.trace.depth {
            self.trace.depth -= 1;
            return;
        }
        self.trace.depth -= 1;
        let stack_len = self.trace.stack.len();
        if let Some(mut trace) = self.trace.stack.pop() {
            if self.trace.value_track_limit == -1
                || self.trace.value_track_limit as usize >= stack_len
            {
                trace.value = value;
            }
            trace.stop = self.position;
            if trace.function != function {
                panic!("{} != {}", trace.function, function);
            }

            let len = self.trace.stack.len();
            if len > 0 {
                self.trace.stack[len - 1].read.push(trace);
            } else {
                self.trace.read.push(trace);
            }
        }
    }

    pub fn read_trace_add_info(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();
        if let Some(t) = self.trace.read.last_mut() {
            t.info.insert(name, value);
        }
    }
}

macro_rules! trace_start {
    ($self:expr, $readahead:ident) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_start(function!(), $readahead);
        }
    };
    ($self:ident, $readahead:ident) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_start(function!(), $readahead);
        }
    };
}
pub(crate) use trace_start;

macro_rules! trace_stop {
    ($self:expr, $value:expr, $valueType:ident) => {
        paste! {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_stop(TraceValue::[< $valueType:upper >]($value), function!());
        }
        }
    };
    ($self:expr, $value:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_stop($value.to_tracevalue(), function!());
        }
    };
    ($self:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_stop(TraceValue::None, function!());
        }
    };
}
pub(crate) use trace_stop;

macro_rules! trace_abort {
    ($self:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_abort();
        }
    };
}
pub(crate) use trace_abort;

macro_rules! trace_annotate {
    ($self:expr, $value:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_annotate($value);
        }
    };
}
pub(crate) use trace_annotate;

macro_rules! trace_info {
    ($self:expr, $name:expr, $value:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_add_info($name, format!("{:?}", $value));
        }
    };
}
pub(crate) use trace_info;

macro_rules! trace_lock {
    ($self:expr) => {
        if $self.trace.enabled {
            assert_eq!($self.trace.locked, false);
            $self.trace.locked = true;
        }
    };
}
pub(crate) use trace_lock;

macro_rules! trace_unlock {
    ($self:expr) => {
        if $self.trace.enabled {
            assert_eq!($self.trace.locked, true);
            $self.trace.locked = false;
        }
    };
}
pub(crate) use trace_unlock;

impl PrintType for TraceValue {
    fn print_type(&self) -> String {
        let rv = match self {
            TraceValue::None => "None".to_string(),
            TraceValue::VecU8(d) => d.print_type().clone(),
            TraceValue::U8(d) => d.print_type().clone(),
            TraceValue::U16(d) => d.print_type().clone(),
            TraceValue::U32(d) => d.print_type().clone(),
            TraceValue::I8(d) => d.print_type().clone(),
            TraceValue::I16(d) => d.print_type().clone(),
            TraceValue::I32(d) => d.print_type().clone(),
            TraceValue::F32(d) => d.print_type().clone(),
            TraceValue::ServerMessage(d) => d.print_type().clone(),
            TraceValue::Packet(d) => d.print_type().clone(),
            TraceValue::StringByte(d) => d.print_type().clone(),
            TraceValue::DeltaUserCommand(d) => d.print_type().clone(),
            TraceValue::StringVector(d) => d.print_type().clone(),
            TraceValue::MvdFrame(d) => d.print_type().clone(),
        };
        rv.to_owned()
    }
}

macro_rules! create_trace_enums{
    ($(($ty:ident, $en:ident)), *) => {
        paste! {
            #[derive(Debug, Default, PartialEq, PartialOrd, Display, Serialize, Clone)]
            pub enum TraceValue{
                #[default] None,
                VecU8(Vec<u8>),
                $(
                [< $en >]([< $ty >]),
                )*
            }

            $(
                impl ToTraceValue for $ty {
                    fn to_tracevalue(&self) -> TraceValue {
                        TraceValue::[< $en >](self.clone())
                    }
                }
                impl PrintType for $ty {
                    fn print_type(&self) -> String {
                        stringify!($en).to_string()
                    }
                }
                )*
        }
    };
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
pub(crate) use function;

impl ToTraceValue for Vec<u8> {
    fn to_tracevalue(&self) -> TraceValue {
        TraceValue::VecU8(self.clone())
    }
}

impl PrintType for Vec<u8> {
    fn print_type(&self) -> String {
        "Vec<u8>".to_owned()
    }
}

create_trace_enums!(
    (u8, U8),
    (u16, U16),
    (u32, U32),
    (i8, I8),
    (i16, I16),
    (i32, I32),
    (f32, F32),
    (ServerMessage, ServerMessage),
    (Packet, Packet),
    (StringByte, StringByte),
    (DeltaUserCommand, DeltaUserCommand),
    (StringVector, StringVector),
    (MvdFrame, MvdFrame)
);
