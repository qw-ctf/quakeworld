use std::{convert::Infallible, fmt::Display, io::Write, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("read error")]
    ParseError,
    #[error("node not found")]
    NodeNotFoundError,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("node ({0}) not found")]
    NodeHashNotFoun(String),
    #[error("file ({0}) not found")]
    FileNotFound(super::VfsQueryFile),
    #[error("pak error: {0}")]
    PakError(#[from] crate::pak::Error),
    #[error("infallible: {0}")]
    InfallibleError(#[from] Infallible),
}

pub type Result<T> = core::result::Result<T, Error>;
