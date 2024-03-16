use paste::paste;
use serde::Serialize;

use std::io::{Cursor, Read};
use thiserror::Error;

#[cfg(feature = "trace")]
use crate::trace::{trace_start, trace_stop, Trace};

use super::common::AsciiString;

#[derive(Error, Debug)]
pub enum DataTypeReaderError {
    #[error("read error")]
    ReadError,
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("read size error: index({0}) length({1}) read_size({2})")]
    ReadSizeError(u64, u64, u64),
    #[error("bound error: start({0}) length({1}) is ({2}) beyond ({3})")]
    BoundCheckError(u64, u64, u64, u64),
}
// DataTypeReader: implements a generic parser for structs
pub struct DataTypeReader<'a> {
    pub data: Vec<u8>,
    pub cursor: Cursor<Vec<u8>>,
    #[cfg(feature = "trace")]
    pub trace: Option<&'a mut Trace>,
}

impl<'a> DataTypeReader<'a> {
    pub fn new(data: Vec<u8>, #[cfg(feature = "trace")] trace: Option<&'a mut Trace>) -> Self {
        DataTypeReader {
            data: data.clone(),
            cursor: Cursor::new(data),
            #[cfg(feature = "trace")]
            trace,
        }
    }
    pub fn read_exact(&mut self, buf: &mut Vec<u8>) -> Result<(), DataTypeReaderError> {
        let n = buf.len();
        trace_start!(self, format!("Vec<u8>[{}", n));
        for i in 0..n {
            buf[i] = <u8 as DataTypeRead>::read(self)?;
        }
        Ok(())
    }
}

pub trait DataTypeRead: Sized {
    fn read(datatypereader: &mut DataTypeReader) -> Result<Self, DataTypeReaderError>;
}

// Bound checking trait
pub trait DataTypeBoundCheck {
    fn check_bounds(&self, datatypereader: &mut DataTypeReader) -> Result<(), DataTypeReaderError>;
}

macro_rules! datatypereader_generate_base_type {
    ($($ty:ty), *) => {
        $(
        paste! {
        impl DataTypeRead for $ty {
        fn  [< read >] (datareader: &mut DataTypeReader,
        ) ->  Result<$ty, DataTypeReaderError> {
        const TYPE_SIZE:usize = std::mem::size_of::<$ty>();
        let current_position: u64 = datareader.cursor.position();
        #[cfg(feature = "trace")]
        trace_start!(datareader, stringify!($ty));
        let len = datareader.cursor.get_ref().len() as u64;
        if (current_position + TYPE_SIZE as u64) > len {
        return Err(DataTypeReaderError::ReadSizeError(current_position, len, TYPE_SIZE as u64));
        }
        let mut a: [u8; TYPE_SIZE] = [0; TYPE_SIZE];
        match datareader.cursor.read_exact(&mut a) {
        Err(_) => {
        return Err(DataTypeReaderError::ReadError)
        }
        Ok(_) => {},
        };

        let v;
        v = $ty::from_le_bytes(a);
        trace_stop!(datareader, v, $ty);
        Ok(v)
        }
        }
        }
        )*
    }
}

macro_rules! datatypereader_generate_sized {
    ($(($ty:tt, $size:expr, $default: expr, $typename: expr)),*) => {
        $(
        datatypereader_generate_sized_dispatch!($ty, $size, $default, $typename);
        )*
    };
}

macro_rules! datatypereader_generate_sized_dispatch {
    (u8, $size: expr, $default: expr, $typename: expr) => {
        paste! {
        impl AsciiString for $typename {
                fn ascii_string(&self) -> String {
                    let owned = self.0.to_owned();
                    let conv = String::from_utf8_lossy(&owned);
                    conv.chars().filter(|&c| c != '\0').collect()
                }
            }
        }
        datatypereader_generate_sized_dispatch_general!(u8, $size, $default, $typename);
    };
    ($ty:ty, $size: expr, $default: expr, $typename: expr) => {
        datatypereader_generate_sized_dispatch_general!($ty, $size, $default, $typename);
    };
}

macro_rules! datatypereader_generate_sized_dispatch_general {
    ($ty:ty, $size: expr, $default: expr, $typename: expr) => {
        paste! {
        #[derive(Serialize, Debug,  Clone, Default)]
        pub struct $typename(pub Vec<$ty>);
        impl DataTypeRead for $typename {
        fn  [< read >] (datareader: &mut DataTypeReader,
        ) ->  Result<$typename, DataTypeReaderError> {
        const TYPE_SIZE:usize = std::mem::size_of::<$ty>();
        let current_position: u64 = datareader.cursor.position();
        #[cfg(feature = "trace")]
        trace_start!(datareader, stringify!($ty));
        let len = datareader.cursor.get_ref().len() as u64;
        if (current_position + TYPE_SIZE as u64 * $size) > len {
        return Err(DataTypeReaderError::ReadSizeError(current_position, len, TYPE_SIZE as u64));
        }
        let mut ret: Vec<$ty> = vec![];
        for _ in 0..$size {
        let mut a: [u8; TYPE_SIZE] = [0; TYPE_SIZE];
        match datareader.cursor.read_exact(&mut a) {
        Err(_) => {
        return Err(DataTypeReaderError::ReadError)
        }
        Ok(_) => {},
        };

        let v;
        v = $ty::from_le_bytes(a);

        ret.push(v);
        }
        trace_stop!(datareader, ret, $ty);
        Ok($typename(ret))
        }
        }
        }
    };
}

// generate read function for base types
datatypereader_generate_base_type!(u8, u16, u32, i8, i16, i32, f32);
// generate read functions for sized types
datatypereader_generate_sized!((u8, 56, 0, PakFileName), (u8, 16, 0, MdlFrameName));
