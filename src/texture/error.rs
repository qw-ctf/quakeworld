use png::EncodingError;
use strum_macros::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("png error")]
    Encoding(#[from] png::EncodingError),
    #[error("palette error")]
    Palette(#[from] crate::lmp::PaletteError),
    #[error("index into texture {0} > {1}")]
    AtlasIndexTexture(usize, usize),
    #[error("index into atlas texture {0} > {1}")]
    AtlasIndexAtlasTexture(usize, usize),
}

pub type Result<T> = core::result::Result<T, Error>;
