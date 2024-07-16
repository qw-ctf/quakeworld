use crate::protocol::errors::MvdParseError;
use crate::protocol::message::Message;
use crate::protocol::message::MessageFlags;
use crate::protocol::message::MessageType;
use crate::protocol::types::*;
use serde::Serialize;

#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;

use crate::protocol::message::trace::*;

#[derive(Serialize, Clone, PartialEq, Eq, Debug, PartialOrd)]
pub struct MvdTarget {
    pub to: u32,
    pub command: DemoCommand,
}

impl Default for MvdTarget {
    fn default() -> Self {
        MvdTarget {
            to: 0,
            command: DemoCommand::Empty,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Mvd {
    pub size: usize,
    pub finished: bool,
    pub message: Message,
    pub last: MvdTarget,
    pub frame: u32,
    pub time: f64,
    #[cfg(feature = "trace")]
    pub trace: bool,
    #[cfg(feature = "trace")]
    pub trace_value_depth: u32,
}

#[derive(Serialize, Clone, PartialEq, Debug, PartialOrd)]
pub struct MvdFrame {
    pub messages: Vec<ServerMessage>,
    pub frame: u32,
    pub time: f64,
    pub last: MvdTarget,
}

impl MvdFrame {
    pub fn empty() -> MvdFrame {
        MvdFrame {
            messages: vec![],
            frame: 0,
            time: 0.0,
            last: MvdTarget {
                ..Default::default()
            },
        }
    }
}

impl Mvd {
    pub fn empty() -> Mvd {
        Mvd {
            size: 0,
            message: Message::empty(),
            finished: false,
            last: MvdTarget {
                ..Default::default()
            },
            frame: 0,
            time: 0.0,
            #[cfg(feature = "trace")]
            trace: false,
            #[cfg(feature = "trace")]
            trace_value_depth: 0,
        }
    }

    pub fn new(
        buffer: Vec<u8>,
        #[cfg(feature = "ascii_strings")] maybe_ascii_converter: Option<AsciiConverter>,
        #[cfg(feature = "trace")] trace: bool,
        #[cfg(feature = "trace")] trace_value_depth: u32,
    ) -> Result<Mvd, std::io::Error> {
        let buffer_heap = Box::new(buffer.clone());

        let mut message = Message::new(
            buffer_heap,
            0,
            buffer.len(),
            false,
            MessageFlags::new_empty(),
            #[cfg(feature = "ascii_strings")]
            maybe_ascii_converter,
            MessageType::Mvd,
        );

        #[cfg(feature = "trace")]
        {
            message.trace.enabled = trace;
        }

        Ok(Mvd {
            size: buffer.len(),
            message,
            finished: false,
            last: MvdTarget {
                ..Default::default()
            },
            frame: 0,
            time: 0.0,
            #[cfg(feature = "trace")]
            trace,
            #[cfg(feature = "trace")]
            trace_value_depth,
        })
    }

    pub fn parse_frame(&mut self) -> Result<Box<MvdFrame>, MvdParseError> {
        let mut frame = Box::new(MvdFrame::empty());
        frame.frame = self.frame;
        self.frame += 1;

        trace_annotate!(self.message, format!("({:.4})({})", self.time, self.frame));
        trace_start!(self.message, false);
        trace_annotate!(self.message, "demo_time");
        let demo_time = self.message.read_u8(false)?;
        trace_info!(self.message, "demo_time", demo_time);
        self.time += demo_time as f64 * 0.001;
        frame.time = self.time;

        trace_annotate!(self.message, "cmd");
        let cmd = self.message.read_u8(false)?;
        let msg_type_try = DemoCommand::try_from(cmd & 7);
        let msg_type = match msg_type_try {
            Ok(msg_type) => msg_type,
            Err(_) => return Err(MvdParseError::UnhandledCommand(cmd & 7)),
        };

        trace_info!(self.message, "cmd", msg_type);
        if msg_type == DemoCommand::Command {
            return Err(MvdParseError::QwdCommand);
        }
        frame.last.command = msg_type;
        if msg_type >= DemoCommand::Multiple && msg_type <= DemoCommand::All {
            match msg_type {
                DemoCommand::Multiple => {
                    trace_annotate!(self.message, "last_to");
                    self.last.to = self.message.read_u32(false)?;
                    self.last.command = msg_type;
                }
                DemoCommand::Single => {
                    self.last.to = (cmd >> 3) as u32;
                    self.last.command = msg_type;
                }
                DemoCommand::All => {
                    self.last.to = 0;
                    self.last.command = msg_type;
                }
                DemoCommand::Stats => {
                    self.last.to = (cmd >> 3) as u32;
                    self.last.command = msg_type;
                }
                DemoCommand::Command => {}
                DemoCommand::Empty => {}
                DemoCommand::Set => {
                    // incoming
                    trace_annotate!(self.message, "sequence");
                    let _ = self.message.read_u32(false);
                    // outgoing
                    trace_annotate!(self.message, "sequence_ack");
                    let _ = self.message.read_u32(false);
                    trace_stop!(self.message, frame);
                    return Ok(frame);
                }
                DemoCommand::Read => {}
            }
        }
        let mut loop_read_packet = true;
        while loop_read_packet {
            loop_read_packet = self.read_packet(&mut frame)?;
        }
        trace_stop!(self.message, frame);
        Ok(frame)
    }

    pub fn read_svc(&mut self, frame: &mut Box<MvdFrame>) -> Result<bool, MvdParseError> {
        trace_annotate!(self.message, "message_cmd");
        trace_start!(self.message, false);
        let msg_cmd = self.message.read_u8(false)?;

        // handle EndOfDemo
        if msg_cmd == 69 {
            let s = self.message.read_stringbyte(true)?;
            if String::from_utf8_lossy(&s.bytes) == *"ndOfDemo" {
                self.finished = true;
                trace_stop!(self.message);
                return Ok(false);
            }
        }
        let cmd = match ServerClient::try_from(msg_cmd) {
            Ok(cmd) => cmd,
            Err(_) => {
                trace_stop!(self.message);
                return Err(MvdParseError::UnhandledCommand(msg_cmd));
            }
        };

        trace_annotate!(self.message, "message");
        let ret = cmd.read_message(&mut self.message)?;

        match ret {
            ServerMessage::Serverdata(r) => {
                frame.messages.push(ServerMessage::Serverdata(r.clone()));
                self.message.flags.fte_protocol_extensions = r.fte_protocol_extension;
                self.message.flags.fte_protocol_extensions_2 = r.fte_protocol_extension_2;
            }
            _ => {
                frame.messages.push(ret);
            }
        }
        return Ok(true);
    }
    pub fn read_packet(&mut self, frame: &mut Box<MvdFrame>) -> Result<bool, MvdParseError> {
        trace_start!(self.message, false);
        trace_annotate!(self.message, "size");
        let size = self.message.read_u32(false)? as usize;
        if size == 0 {
            trace_stop!(self.message);
            return Ok(false);
        }

        if self.last.command == DemoCommand::Multiple && self.last.to == 0 {
            self.message.position += size;
            trace_stop!(self.message);
            return Ok(false);
        }

        let message_start = self.message.position;
        loop {
            if self.message.position >= message_start + size {
                break;
            }
            trace_annotate!(self.message, "message_cmd");
            let msg_cmd = self.message.read_u8(false)?;

            // handle EndOfDemo
            if msg_cmd == 69 {
                let s = self.message.read_stringbyte(true)?;
                if String::from_utf8_lossy(&s.bytes) == *"ndOfDemo" {
                    self.finished = true;
                    trace_stop!(self.message);
                    return Ok(false);
                }
            }
            let cmd = match ServerClient::try_from(msg_cmd) {
                Ok(cmd) => cmd,
                Err(_) => {
                    trace_stop!(self.message);
                    return Err(MvdParseError::UnhandledCommand(msg_cmd));
                }
            };

            trace_annotate!(self.message, "message");
            let ret = cmd.read_message(&mut self.message)?;

            match ret {
                ServerMessage::Serverdata(r) => {
                    frame.messages.push(ServerMessage::Serverdata(r.clone()));
                    self.message.flags.fte_protocol_extensions = r.fte_protocol_extension;
                    self.message.flags.fte_protocol_extensions_2 = r.fte_protocol_extension_2;
                }
                _ => {
                    frame.messages.push(ret);
                }
            }

            if self.message.position >= self.message.length {
                break;
            }
        }

        trace_stop!(self.message);
        Ok(false)
    }
}
