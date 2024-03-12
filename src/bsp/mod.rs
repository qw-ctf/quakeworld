use crate::datatypes::bsp;
use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use serde::Serialize;
use thiserror::Error;

#[cfg(feature = "trace")]
use crate::trace::Trace;

#[derive(Error, Debug)]
pub enum BspError {
    #[error("read error")]
    ReadError,
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("io error {0}")]
    IoError(std::io::Error),
    #[error("datatypereader error: {0}")]
    DataTypeReaderError(DataTypeReaderError),
}

impl From<DataTypeReaderError> for BspError {
    fn from(err: DataTypeReaderError) -> BspError {
        BspError::DataTypeReaderError(err)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Bsp {
    header: bsp::BspHeader,
}

impl Bsp {
    pub fn parse(
        data: Vec<u8>,
        #[cfg(feature = "trace")] trace: Option<&mut Trace>,
    ) -> Result<Self, BspError> {
        let mut datatypereader = DataTypeReader::new(
            data,
            #[cfg(feature = "trace")]
            trace,
        );
        let header = <bsp::BspHeader as DataTypeRead>::read(&mut datatypereader)?;
        header.check_bounds(&mut datatypereader)?;

        Ok(Bsp { header })
    }
}
