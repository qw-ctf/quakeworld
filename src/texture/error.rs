use png::EncodingError;
use strum_macros::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
pub enum Error {
    Io(#[from] std::io::Error),
    Encoding(#[from] png::EncodingError),
    Palette(#[from] crate::lmp::PaletteError),
}

pub type Result<T> = core::result::Result<T, Error>;
