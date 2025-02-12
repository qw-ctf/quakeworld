use std::io::prelude::*;
use std::io::Cursor;
use std::io::SeekFrom;

use protocol_macros::DataTypeBoundCheckDerive;
use serde::Serialize;
use thiserror::Error;

use crate::datatypes::pak;
use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};

#[cfg(feature = "trace")]
use crate::trace::Trace;

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
    #[error("datareadererror: {0}")]
    DataTypeReaderError(DataTypeReaderError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<DataTypeReaderError> for Error {
    fn from(err: DataTypeReaderError) -> Error {
        Error::DataTypeReaderError(err)
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

static HEADER: u32 = 0x4b434150; // PACK
pub const MAX_NAME_LENGTH: usize = 55;
const NAME_LENGTH: u32 = 56;

pub type File = pak::File;

#[derive(Serialize, Debug, Default, Clone, DataTypeBoundCheckDerive)]
pub struct Pak {
    pub name: String,
    pub data: Vec<u8>,
    #[check_bounds]
    pub files: Vec<pak::File>,
}

type Result<T> = core::result::Result<T, Error>;

impl Pak {
    pub fn load(
        name: impl Into<String>,
        mut reader: impl Read,
        #[cfg(feature = "trace")] trace: Option<&mut Trace>,
    ) -> Result<Pak> {
        let mut data = Vec::new();
        match reader.read_to_end(&mut data) {
            Ok(size) => size,
            Err(err) => return Err(Error::Io(err)),
        };
        Pak::parse(
            name,
            data,
            #[cfg(feature = "trace")]
            trace,
        )
    }

    pub fn parse(
        name: impl Into<String>,
        data: impl Into<Vec<u8>>,
        #[cfg(feature = "trace")] trace: Option<&mut Trace>,
    ) -> Result<Pak> {
        let name = name.into();
        let data = data.into();
        let mut datatypereader = DataTypeReader::new(
            data.clone(),
            #[cfg(feature = "trace")]
            trace,
        );
        let header = <pak::Header as DataTypeRead>::read(&mut datatypereader)?;
        header.check_bounds(&mut datatypereader)?;

        if header.version != HEADER {
            return Err(Error::HeaderMismatch(header.version, HEADER));
        }

        let file_count = header.directory_offset.size / (NAME_LENGTH + 4 * 2);

        datatypereader
            .cursor
            .seek(SeekFrom::Start((header.directory_offset.offset).into()))?;
        let mut files = Vec::new();
        for _ in 0..file_count {
            let f = <pak::File as DataTypeRead>::read(&mut datatypereader)?;
            files.push(f);
        }
        let p = Pak { name, data, files };
        p.check_bounds(&mut datatypereader)?;

        Ok(p)
    }

    pub fn get_data(&self, file: &pak::File) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(&self.data);
        let size: usize = file.size.try_into()?;
        let mut buf = vec![0; size];
        cursor.seek(SeekFrom::Start(file.offset as u64))?;
        cursor.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[derive(Serialize, Debug)]
pub struct PakWriter {
    files: Vec<PakWriterFile>,
}

#[derive(Serialize, Debug)]
struct PakWriterFile {
    name: Vec<u8>,
    data: Vec<u8>,
}

impl PakWriterFile {
    pub fn name_as_string(&self) -> String {
        // @FIXME:  handle this unwrap and all the other crap
        let s = String::from_utf8(self.name.clone()).unwrap();
        let s = s.trim_matches(char::from(0));
        s.to_string()
    }
}

impl Default for PakWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl PakWriter {
    pub fn new() -> PakWriter {
        PakWriter { files: Vec::new() }
    }

    pub fn file_add(&mut self, name: Vec<u8>, mut data: impl Read) -> Result<()> {
        if name.len() > MAX_NAME_LENGTH {
            return Err(Error::MaxNameLength(name.len(), MAX_NAME_LENGTH));
        }
        let mut file_data = Vec::new();

        data.read_to_end(&mut file_data)?;
        self.files.push(PakWriterFile {
            name,
            data: file_data,
        });
        Ok(())
    }

    pub fn write_data(self) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut c = Cursor::new(&mut buffer);
        c.write_all(&HEADER.to_le_bytes())?;
        let dir_offset: u32 = 4 * 3;
        c.write_all(&dir_offset.to_le_bytes())?;
        let dir_size: u32 = (self.files.len() * (MAX_NAME_LENGTH + 1 + 8)) as u32;
        c.write_all(&dir_size.to_le_bytes())?;
        let mut file_position = dir_offset + dir_size;
        for file in &self.files {
            let mut name_buffer: [u8; NAME_LENGTH as usize] = [0; NAME_LENGTH as usize];
            name_buffer[..file.name.len()].copy_from_slice(&file.name);
            c.write_all(&name_buffer)?;
            c.write_all(&file_position.to_le_bytes())?;
            c.write_all(&(file.data.len() as u32).to_le_bytes())?;
            file_position += file.data.len() as u32;
        }
        for file in &self.files {
            c.write_all(&file.data)?;
        }
        Ok(buffer)
    }
}

#[macro_export]
macro_rules! create_pak {
    ($(($name: expr, $data: expr)), *) => {{
        let mut pak = PakWriter::new();
        $(
            pak.file_add($name.to_string().into(), &$data[..]);
        )*
        pak
    }};
}

#[cfg(test)]
mod tests {
    use crate::pak::PakWriter;
    #[test]
    pub fn pak_creation_and_reading() -> Result<(), crate::pak::Error> {
        const FILE1_NAME: &str = "dir/file1";
        const FILE1_DATA: &[u8; 8] = b"01234567";
        const FILE2_NAME: &str = "dir_a/file2";
        const FILE2_DATA: &[u8; 8] = b"76543210";
        let pack = create_pak!((FILE1_NAME, FILE1_DATA), (FILE2_NAME, FILE2_DATA));

        // check if the files are properly inserted
        assert_eq!(pack.files[0].name_as_string(), FILE1_NAME);
        assert_eq!(pack.files[0].data, FILE1_DATA.to_vec());
        assert_eq!(pack.files[1].name_as_string(), FILE2_NAME);
        assert_eq!(pack.files[1].data, FILE2_DATA.to_vec());

        let data = pack.write_data()?;

        //reread the pak
        let read_pack = crate::pak::Pak::parse(
            "my_pak",
            data,
            #[cfg(feature = "trace")]
            None,
        )?;
        assert_eq!(2, read_pack.files.len());
        // names
        assert_eq!(FILE1_NAME, read_pack.files[0].name_as_string());
        assert_eq!(FILE2_NAME, read_pack.files[1].name_as_string());
        // data size
        assert_eq!(FILE1_DATA.to_vec().len() as u32, read_pack.files[0].size);
        assert_eq!(FILE2_DATA.to_vec().len() as u32, read_pack.files[1].size);
        // data
        assert_eq!(
            FILE1_DATA.to_vec(),
            read_pack.get_data(&read_pack.files[0])?
        );
        assert_eq!(
            FILE2_DATA.to_vec(),
            read_pack.get_data(&read_pack.files[1])?
        );
        return Ok(());
    }
}
