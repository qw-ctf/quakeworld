use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

use crate::datatypes::reader::Error as ReaderError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("header mismath: {0} != {1}")]
    HeaderMismatch(u32, u32),
    #[error("io {0}")]
    Io(std::io::Error),
    #[error("from utf8 {0}")]
    UtfConversion(std::string::FromUtf8Error),
    #[error("try from int {0}")]
    IntConversion(std::num::TryFromIntError),
    #[error("supplied file name is longer than {0} > {1}")]
    MaxNameLength(usize, usize),
    #[error("write length mismatch expected: {0}, got: {1}")]
    WriteLength(usize, usize),
    #[error("reader error: {0}")]
    Reader(ReaderError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<ReaderError> for Error {
    fn from(err: ReaderError) -> Error {
        Error::Reader(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::UtfConversion(err)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(err: std::num::TryFromIntError) -> Error {
        Error::IntConversion(err)
    }
}
