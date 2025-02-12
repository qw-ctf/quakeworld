use crate::datatypes::bsp;
use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};
use serde::Serialize;
use thiserror::Error;

#[cfg(feature = "trace")]
use crate::trace::Trace;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io {0}")]
    Io(std::io::Error),
    #[error("datatypereader error: {0}")]
    DataTypeReader(DataTypeReaderError),
}

impl From<DataTypeReaderError> for Error {
    fn from(err: DataTypeReaderError) -> Error {
        Error::DataTypeReader(err)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Bsp {
    header: bsp::Header,
}

pub type Result<T> = core::result::Result<T, Error>;

impl Bsp {
    pub fn parse(
        data: Vec<u8>,
        #[cfg(feature = "trace")] trace: Option<&mut Trace>,
    ) -> Result<Self> {
        let mut datatypereader = DataTypeReader::new(
            data,
            #[cfg(feature = "trace")]
            trace,
        );
        let header = <bsp::Header as DataTypeRead>::read(&mut datatypereader)?;
        header.check_bounds(&mut datatypereader)?;

        Ok(Bsp { header })
    }
}
