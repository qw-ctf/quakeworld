use serde::Serialize;
use thiserror::Error;

use super::reader::Error as ReaderError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("read error")]
    Read,
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error {0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Reader(ReaderError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<ReaderError> for Error {
    fn from(err: ReaderError) -> Error {
        Error::Reader(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}
