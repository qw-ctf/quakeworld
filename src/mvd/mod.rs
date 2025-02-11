use std::thread;

use crate::protocol::errors::MvdParseError;
use crate::protocol::message::Message;
use crate::protocol::message::MessageFlags;
use crate::protocol::message::MessageType;
use crate::protocol::types::*;
use serde::Serialize;

#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;

use crate::protocol::message::trace;

#[derive(Serialize, Clone, PartialEq, Eq, Debug, PartialOrd, Default)]
pub struct MvdFrameIndex {
    start: usize,
    stop: usize,
}

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
    pub serverdata_read: bool,
    #[cfg(feature = "trace")]
    pub trace_options: TraceOptions,
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
            serverdata_read: false,
            #[cfg(feature = "trace")]
            trace_options: TraceOptions::default(),
        }
    }

    pub fn new(
        buffer: Vec<u8>,
        #[cfg(feature = "ascii_strings")] maybe_ascii_converter: Option<AsciiConverter>,
        #[cfg(feature = "trace")] trace_options: TraceOptions,
    ) -> Result<Mvd, std::io::Error> {
        let buffer_heap = Box::new(buffer.clone());

        let message = Message::new(
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
            message.trace.enabled = trace_options.enabled;
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
            serverdata_read: false,
            #[cfg(feature = "trace")]
            trace_options,
        })
    }

    pub fn parse_mutlithreaded(
        &mut self,
        thread_count: usize,
    ) -> Result<Vec<MvdFrame>, MvdParseError> {
        // read frames till we hit the protocol stuff
        while !self.serverdata_read {
            let _ = self.parse_frame();
        }
        println!("got protocols on frame: {}", self.frame - 1);
        let rval = vec![];
        // get the indexes for the rest of the frames
        let frame_indexes: Vec<_> = self.get_frame_indexes()?;
        let chunk_size = frame_indexes.len() / thread_count;
        let chunks: Vec<&[MvdFrameIndex]> = frame_indexes.chunks(chunk_size).collect();

        let mut handles = vec![];
        for (index, chunk) in chunks.iter().enumerate() {
            // let c: Vec<MvdFrameIndex> = chunk.to_vec();
            let mut mtf = MvdThreadData {
                chunk: chunk.to_vec(),
                data: self.message.buffer.clone(),
                message_flags: self.message.flags,
                index: index as u32,
            };
            let handle = thread::spawn(move || -> MvdThreadReturn { mtf.parse_frame_chunk() });

            handles.push(handle);
        }
        for handle in handles {
            if let Ok(v) = handle.join() {
                println!("index: {} frames: {}", v.index, v.frames.len());
            };
        }
        Ok(rval)
    }

    pub fn get_frame_indexes(&mut self) -> Result<Vec<MvdFrameIndex>, MvdParseError> {
        let mut rval: Vec<MvdFrameIndex> = vec![];
        let start: usize = 0;
        let stop: usize = 0;
        let mut i = 0;
        while self.message.position < self.message.length {
            i += 1;
            let mut f = MvdFrameIndex {
                start: self.message.position,
                stop: self.message.position,
            };
            let _demo_time = self.message.read_u8(false)?;
            let cmd = self.message.read_u8(false)?;
            let msg_type_try = DemoCommand::try_from(cmd & 7);
            let msg_type = match msg_type_try {
                Ok(msg_type) => msg_type,
                Err(_) => return Err(MvdParseError::UnhandledCommand(cmd & 7)),
            };

            if msg_type >= DemoCommand::Multiple && msg_type <= DemoCommand::All {
                match msg_type {
                    DemoCommand::Multiple => {
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
                        let _ = self.message.read_u32(false);
                        // outgoing
                        let _ = self.message.read_u32(false);
                        f.stop = self.message.position;
                        rval.push(f);
                        break;
                    }
                    DemoCommand::Read => {}
                }
            }
            let _loop_read_packet = true;
            let mut _p = 0;
            let message_start = self.message.position;
            loop {
                _p += 1;
                let size = self.message.read_u32(false)? as usize;
                if size == 0 {
                    f.stop = self.message.position;
                    rval.push(f);
                    break;
                }
                self.message.position += size;

                if self.last.command == DemoCommand::Multiple && self.last.to == 0 {
                    f.stop = self.message.position;
                    rval.push(f);
                    break;
                }

                if self.message.position >= message_start + size {
                    f.stop = self.message.position;
                    rval.push(f);
                    break;
                }
            }
        }
        Ok(rval)
    }

    pub fn parse_frame(&mut self) -> Result<Box<MvdFrame>, MvdParseError> {
        let mut frame = Box::new(MvdFrame::empty());
        frame.frame = self.frame;
        self.frame += 1;

        trace::trace_annotate!(self.message, format!("({:.4})({})", self.time, self.frame));
        trace::trace_start!(self.message, false);
        trace::trace_annotate!(self.message, "demo_time");
        let demo_time = self.message.read_u8(false)?;
        trace::trace_info!(self.message, "demo_time", demo_time);
        self.time += demo_time as f64 * 0.001;
        frame.time = self.time;

        trace::trace_annotate!(self.message, "cmd");
        let cmd = self.message.read_u8(false)?;
        let msg_type_try = DemoCommand::try_from(cmd & 7);
        let msg_type = match msg_type_try {
            Ok(msg_type) => msg_type,
            Err(_) => return Err(MvdParseError::UnhandledCommand(cmd & 7)),
        };

        trace::trace_info!(self.message, "cmd", msg_type);
        if msg_type == DemoCommand::Command {
            return Err(MvdParseError::QwdCommand);
        }
        frame.last.command = msg_type;
        if msg_type >= DemoCommand::Multiple && msg_type <= DemoCommand::All {
            match msg_type {
                DemoCommand::Multiple => {
                    trace::trace_annotate!(self.message, "last_to");
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
                    trace::trace_annotate!(self.message, "sequence");
                    let _ = self.message.read_u32(false);
                    // outgoing
                    trace::trace_annotate!(self.message, "sequence_ack");
                    let _ = self.message.read_u32(false);
                    trace::trace_stop!(self.message, frame);
                    return Ok(frame);
                }
                DemoCommand::Read => {}
            }
        }
        let mut loop_read_packet = true;
        while loop_read_packet {
            loop_read_packet = self.read_packet(&mut frame)?;
        }
        trace::trace_stop!(self.message, frame);
        Ok(frame)
    }

    pub fn read_svc(&mut self, frame: &mut Box<MvdFrame>) -> Result<bool, MvdParseError> {
        trace::trace_annotate!(self.message, "message_cmd");
        trace::trace_start!(self.message, false);
        let msg_cmd = self.message.read_u8(false)?;

        // handle EndOfDemo
        if msg_cmd == 69 {
            let s = self.message.read_stringbyte(true)?;
            if String::from_utf8_lossy(&s.bytes) == *"ndOfDemo" {
                self.finished = true;
                trace::trace_stop!(self.message);
                return Ok(false);
            }
        }
        let cmd = match ServerClient::try_from(msg_cmd) {
            Ok(cmd) => cmd,
            Err(_) => {
                trace::trace_stop!(self.message);
                return Err(MvdParseError::UnhandledCommand(msg_cmd));
            }
        };

        trace::trace_annotate!(self.message, "message");
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
        Ok(true)
    }

    pub fn read_packet(&mut self, frame: &mut Box<MvdFrame>) -> Result<bool, MvdParseError> {
        trace::trace_start!(self.message, false);
        trace::trace_annotate!(self.message, "size");
        let size = self.message.read_u32(false)? as usize;
        if size == 0 {
            trace::trace_stop!(self.message);
            return Ok(false);
        }

        if self.last.command == DemoCommand::Multiple && self.last.to == 0 {
            self.message.position += size;
            trace::trace_stop!(self.message);
            return Ok(false);
        }

        let message_start = self.message.position;
        loop {
            if self.message.position >= message_start + size {
                break;
            }
            trace::trace_annotate!(self.message, "message_cmd");
            let msg_cmd = self.message.read_u8(false)?;

            // handle EndOfDemo
            if msg_cmd == 69 {
                let s = self.message.read_stringbyte(true)?;
                if String::from_utf8_lossy(&s.bytes) == *"ndOfDemo" {
                    self.finished = true;
                    trace::trace_stop!(self.message);
                    return Ok(false);
                }
            }
            let cmd = match ServerClient::try_from(msg_cmd) {
                Ok(cmd) => cmd,
                Err(_) => {
                    trace::trace_stop!(self.message);
                    return Err(MvdParseError::UnhandledCommand(msg_cmd));
                }
            };

            trace::trace_annotate!(self.message, "message");
            let ret = cmd.read_message(&mut self.message)?;

            match ret {
                ServerMessage::Serverdata(r) => {
                    self.serverdata_read = true;
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

        trace::trace_stop!(self.message);
        Ok(false)
    }
}

struct MvdThreadData {
    chunk: Vec<MvdFrameIndex>,
    data: Box<Vec<u8>>,
    message_flags: MessageFlags,
    index: u32,
}

#[derive(Debug)]
struct MvdThreadReturn {
    frames: Vec<MvdFrame>,
    index: u32,
}
impl MvdThreadData {
    pub fn parse_frame_chunk(&mut self) -> MvdThreadReturn {
        let mut frames: Vec<MvdFrame> = vec![];
        #[cfg(feature = "trace")]
        let mut mvd = Mvd::new(*self.data.clone(), None, TraceOptions::default()).unwrap();
        #[cfg(not(feature = "trace"))]
        let mut mvd = Mvd::new(*self.data.clone(), None).unwrap();
        mvd.message.flags = self.message_flags;
        for frame in self.chunk.clone() {
            mvd.message.position = frame.start;
            let f = mvd.parse_frame().unwrap();
            frames.push(*f);
        }
        MvdThreadReturn {
            frames,
            index: self.index,
        }
    }
}
