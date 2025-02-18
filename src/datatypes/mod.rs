use std::io::Cursor;

#[cfg(feature = "trace")]
use crate::trace::Trace;

pub mod bsp;
pub mod common;
pub mod mdl;
pub mod pak;
pub mod reader;

mod error;
pub use error::{Error, Result};

pub trait Parse<T, S> {
    #[cfg(feature = "trace")]
    fn parse(cursor: Cursor<S>, trace: &mut Trace) -> Result<T>;
    #[cfg(not(feature = "trace"))]
    fn parse(cursor: Cursor<S>) -> Result<T>;
}
