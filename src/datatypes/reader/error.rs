use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
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
    #[error("not implemented for this type")]
    NotImplemented,
    #[error("environment variable ({0}) not set for ({1})")]
    EnvironmentVariableNotSet(String, String),
    #[error("environment variable ({0})")]
    EnvironmentVariableNotFound(String),
    #[error("DirectoryEntry size mismatch ({0}) ({1}) ({2})")]
    DirectoryEntrySize(usize, usize, usize),
}

pub type Result<T> = std::result::Result<T, Error>;
