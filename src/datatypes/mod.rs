use serde::Serialize;
use std::io::Cursor;
use thiserror::Error;

#[cfg(feature = "trace")]
use crate::trace::Trace;

#[derive(Error, Debug, Serialize)]
pub enum DataTypeError {}

pub trait Parse<T, S> {
    #[cfg(feature = "trace")]
    fn parse(cursor: Cursor<S>, trace: &mut Trace) -> Result<T, DataTypeError>;
    #[cfg(not(feature = "trace"))]
    fn parse(cursor: Cursor<S>) -> Result<T, DataTypeError>;
}
pub mod bsp;
pub mod common;
pub mod mdl;
pub mod pak;
pub mod reader;
