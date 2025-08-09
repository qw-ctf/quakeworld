use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct TraceviewerArgs {
    #[clap(subcommand)]
    pub command: CommandType,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// print trace output
    #[clap(subcommand)]
    Trace(TraceCommandType),
}

#[derive(Debug, Subcommand)]
pub enum TraceCommandType {
    /// trace an mvd file
    Mvd(TraceCommandMvd),
    /// trace an qtv stream
    Qtv(TraceCommandQtv),
    /// trace a pak file
    Pak(TraceCommandPak),
    /// trace a bsp file
    Bsp(TraceCommandBsp),
    /// trace a mdl file
    Mdl(TraceCommandMdl),
}

#[derive(Debug, Args)]
pub struct TraceCommandMdl {
    #[arg(long, default_value = "-1")]
    /// depth at wich to stop recoding values
    pub trace_value_depth: i32,

    #[arg(long, default_value = "-1")]
    /// depth at wich to stop tracing
    pub trace_depth_limit: i32,

    #[arg(short)]
    /// paks to mount
    pub paks: Option<Vec<String>>,

    /// file to trace
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct TraceCommandBsp {
    #[arg(long, default_value = "-1")]
    /// depth at wich to stop recoding values
    pub trace_value_depth: i32,

    #[arg(long, default_value = "-1")]
    /// depth at wich to stop tracing
    pub trace_depth_limit: i32,

    #[arg(short)]
    /// paks to mount
    pub paks: Option<Vec<String>>,

    /// file to trace
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct TraceCommandPak {
    #[arg(long, default_value = "-1")]
    /// depth at wich to stop recoding values
    pub trace_value_depth: i32,

    #[arg(long, default_value = "-1")]
    /// depth at wich to stop tracing
    pub trace_depth_limit: i32,

    /// file to trace
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct TraceCommandMvd {
    #[arg(long, default_value = "0")]
    /// first frame to trace
    pub frame_start: u32,

    #[arg(long, default_value = "0")]
    /// last frame to trace
    pub frame_stop: u32,

    #[arg(long, default_value = "-1")]
    /// depth at wich to stop recoding values
    pub trace_value_depth: i32,

    #[arg(long, default_value = "-1")]
    /// depth at wich to stop tracing
    pub trace_depth_limit: i32,

    /// file to trace
    pub file: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum TraceFlagsProtocol {
    FtexTrans,
    FtexAccurateTimings,
    FtexHlBsp,
    FtexModeldbl,
    FtexEntitydbl,
    FtexEntitydbl2,
    FtexFloatcoords,
    FtexSpawnstatic2,
    FtexPacketentities256,
    FtexChunkedDownloads,
    Fte2Voicechat,
    Mvd1Floatcoords,
    Mvd1Highlagteleport,
}

#[derive(Debug, Args)]
pub struct TraceCommandQtv {
    #[arg(long, default_value = "0")]
    /// first frame to trace
    pub frame_start: u32,

    #[arg(long)]
    /// last frame to trace
    pub frame_stop: Option<u32>,

    /// file to trace
    pub file: PathBuf,
}
