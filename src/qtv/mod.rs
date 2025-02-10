extern crate paste;

use serde::Serialize;
use std::{
    io::{Cursor, Read},
    num::ParseIntError,
};
use strum_macros::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QtvError {
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("from_utf8 error {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("parse int error {0}")]
    FromParseIntError(#[from] ParseIntError),
    #[error("parse float error {0}")]
    FromParseFloatError(#[from] std::num::ParseFloatError),
    #[error("parsing header error: {0}")]
    HeaderParsing(String),
    #[error("header marlformed: {0}")]
    HeaderMalformed(String),
    // #[error("attempting to read beyond demo size({0}) with position({1}) and size({2})")]
    // ReadBeyondSize(usize, usize, usize),
    // #[error("unhandled command ({0})")]
    // UnhandledCommand(u8),
    // #[error("cannot handle qwd command")]
    // QwdCommand,
    // #[error("read error {0}")]
    // MessageError(MessageError),
}

#[derive(Debug, Clone, Serialize, Display)]
pub enum QtvType {
    Header(String),
}

impl From<QtvType> for crate::datatypes::common::DataType {
    fn from(value: QtvType) -> Self {
        crate::datatypes::common::DataType::QTV(value)
    }
}

type QtvResult<T> = Result<T, QtvError>;

use crate::trace::{trace_annotate, trace_start, trace_stop};

#[cfg(feature = "trace")]
use crate::trace::Trace;

#[derive(Default, Debug, PartialEq)]
pub enum ConnectionState {
    #[default]
    None,
    ParsingHeader,
    ParsingConnection,
}

#[derive(Default, Debug)]
pub struct Qtv {
    pub state: ConnectionState,
    pub protocol: f32,
    pub cursor: Cursor<Vec<u8>>,
    pub data: Vec<u8>,
    #[cfg(feature = "trace")]
    pub trace: Option<Trace>,
}
impl Qtv {
    pub fn position(&self) -> u64 {
        self.cursor.position()
    }
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
        let data = data.into();
        Qtv {
            state: ConnectionState::None,
            cursor: Cursor::new(data.clone()),
            data: data.clone(),
            #[cfg(feature = "trace")]
            trace: Some(Trace::new()),
            protocol: 0.0,
        }
    }
    pub fn parse(&mut self) -> QtvResult<()> {
        if self.state == ConnectionState::None {
            if self.data.len() < 6 {
                return Ok(());
            }
            self.parse_header()?
        }
        if self.state == ConnectionState::ParsingConnection {
            self.parse_connection()?
        }
        Ok(())
    }

    pub fn parse_connection(&mut self) -> QtvResult<()> {
        println!("parse connection:");
        let mut a: [u8; 1] = [0; 1];
        match self.cursor.read_exact(&mut a) {
            Ok(_) => {
                println!("{:?}", a);
            }
            Err(e) => {
                return Err(QtvError::IoError(e));
            }
        }
        Ok(())
    }

    pub fn parse_header(&mut self) -> QtvResult<()> {
        let current_position = self.cursor.position();
        let mut header: Vec<u8> = vec![];
        trace_annotate!(self, function!());
        trace_start!(self, "header");
        let mut first_found = false;
        while self.cursor.position() < self.data.len() as u64 {
            let mut a: [u8; 1] = [0; 1];
            match self.cursor.read_exact(&mut a) {
                Ok(_) => {
                    header.push(a[0]);
                    if a[0] == b'\n' {
                        if !first_found {
                            first_found = true;
                        } else {
                            break;
                        }
                    } else {
                        first_found = false;
                    }
                }
                Err(e) => {
                    return Err(QtvError::IoError(e));
                }
            }
        }
        let header = String::from_utf8(header)?;

        trace_stop!(self, QtvType::Header(header.clone()).into());
        let splits: Vec<&str> = header.split("\n").collect();
        for s in splits {
            if s.starts_with("QTVSV ") {
                let protocol: Vec<&str> = s.split(" ").collect();
                if protocol.len() != 2 {
                    return Err(QtvError::HeaderParsing(format!(
                        "malmformed protocol version: {}",
                        s
                    )));
                }
                self.protocol = protocol[1].parse()?;
            } else {
                if s.is_empty() {
                    continue;
                }
                let (header_entry, header_value) = match s.split_once(": ") {
                    Some(value) => value,
                    None => return Err(QtvError::HeaderMalformed(s.to_string())),
                };
                if header_entry == "BEGIN" {
                    self.state = ConnectionState::ParsingConnection;
                }
            }
        }
        Ok(())
    }

    // #[cfg(feature = "trace")]
    // pub fn read_trace_stop(&mut self, value: TraceValue, function: impl Into<String>) {}
}
