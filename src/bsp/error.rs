use crate::datatypes::reader::Error as ReaderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io {0}")]
    Io(std::io::Error),
    #[error("datatypereader error: {0}")]
    Reader(ReaderError),
}

impl From<ReaderError> for Error {
    fn from(err: ReaderError) -> Error {
        Error::Reader(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
