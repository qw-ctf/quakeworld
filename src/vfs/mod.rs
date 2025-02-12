use serde::Serialize;
use std::{convert::Infallible, fmt::Display, io::Write, path::Path};
use thiserror::Error;
use time::OffsetDateTime;

use internal_node::VfsList;

pub mod internal_node;

pub mod path;
use path::VfsPath;

pub mod meta;
use meta::VfsMetaData;

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("read error")]
    ParseError,
    #[error("node not found")]
    NodeNotFoundError,
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("node ({0}) not found")]
    NodeHashNotFoundError(String),
    #[error("file ({0}) not found")]
    FileNotFound(VfsQueryFile),
    #[error("pak error: {0}")]
    PakError(#[from] crate::pak::Error),
    #[error("infallive {0}")]
    InfallibleError(#[from] Infallible),
}

pub type VfsResult<T> = Result<T, VfsError>;

#[derive(Serialize, Debug, Clone)]
pub struct FileEntry<'a> {
    pub file: &'a Path,
    pub location: &'a Path,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Entry {}

pub type VfsHash = String;

#[derive(Default, Debug, Clone)]
pub struct VfsEntryFile {
    pub path: VfsPath,
    pub meta: VfsMetaData,
    // pub nodes: Vec<VfsHash>,
}
impl From<VfsEntryFile> for VfsQueryFile {
    fn from(val: VfsEntryFile) -> Self {
        VfsQueryFile {
            path: val.path.clone(),
        }
    }
}

impl From<&VfsEntryFile> for VfsQueryFile {
    fn from(val: &VfsEntryFile) -> Self {
        VfsQueryFile {
            path: val.path.clone(),
        }
    }
}

impl Display for VfsEntryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let created: OffsetDateTime = self.meta.time.created.into();
        let modified: OffsetDateTime = self.meta.time.modified.into();
        write!(
            f,
            "f({}) size({}) ctime({:?})  mtime({:?})",
            self.path, self.meta.size, created, modified,
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct VfsEntryDirectory {
    pub path: VfsPath,
    pub meta: VfsMetaData,
    // pub nodes: Vec<VfsHash>,
}

impl From<&VfsEntryDirectory> for VfsQueryDirectory {
    fn from(value: &VfsEntryDirectory) -> Self {
        VfsQueryDirectory {
            path: value.path.clone(),
        }
    }
}

impl From<&str> for VfsQueryDirectory {
    fn from(value: &str) -> Self {
        VfsQueryDirectory { path: value.into() }
    }
}

impl Display for VfsEntryDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let created: OffsetDateTime = self.meta.time.created.into();
        let modified: OffsetDateTime = self.meta.time.modified.into();
        write!(
            f,
            "d({}) size({}) ctime({:?})  mtime({:?})",
            self.path, self.meta.size, created, modified,
        )
    }
}

#[derive(Debug, Clone)]
pub enum VfsEntry {
    File(VfsEntryFile),
    Directory(VfsEntryDirectory),
}
impl VfsEntry {
    fn path(&self) -> VfsPath {
        match self {
            VfsEntry::File(vfs_entry_file) => vfs_entry_file.path.clone(),
            VfsEntry::Directory(vfs_entry_directory) => vfs_entry_directory.path.clone(),
        }
    }
    fn path_prefix(&self) -> VfsPath {
        match self {
            VfsEntry::File(vfs_entry_file) => {
                let mut p = vfs_entry_file.path.clone();
                p.push("f");
                p
            }
            VfsEntry::Directory(vfs_entry_directory) => {
                let mut p = vfs_entry_directory.path.clone();
                p.push("d");
                p
            }
        }
    }
}

impl Display for VfsEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VfsEntry::File(vfs_entry_file) => write!(f, "{}", vfs_entry_file),
            VfsEntry::Directory(vfs_entry_directory) => write!(f, "{}", vfs_entry_directory),
        }
    }
}

pub type VfsRawData = Vec<u8>;

#[derive(Default, Debug, Clone)]
pub struct VfsQueryDirectory {
    pub path: VfsPath,
    // pub node: Option<VfsHash>,
}

impl VfsQueryDirectory {
    pub fn new(path: VfsPath, _node: Option<VfsHash>) -> Self {
        Self { path }
    }
}

impl Display for VfsQueryDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "qd ({})", self.path)
    }
}

impl From<String> for VfsQueryDirectory {
    fn from(value: String) -> Self {
        VfsQueryDirectory {
            path: VfsPath::new(&value).unwrap(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct VfsQueryFile {
    pub path: VfsPath,
    // pub node: Option<VfsHash>,
}

impl From<String> for VfsQueryFile {
    fn from(value: String) -> Self {
        VfsQueryFile {
            path: VfsPath::new(&value).unwrap(),
        }
    }
}

impl From<&str> for VfsQueryFile {
    fn from(value: &str) -> Self {
        VfsQueryFile {
            path: VfsPath::new(value).unwrap(),
        }
    }
}

// impl From<&str> for &VfsQueryFile {
//     fn from(value: &str) -> Self {
//         return std::rc::Rc::new(VfsQueryFile {
//             path: VfsPath::new(&value).unwrap(),
//         });
//     }
// }

impl VfsQueryFile {
    pub fn new(path: VfsPath) -> Self {
        Self { path }
    }
}
impl Display for VfsQueryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "qf ({})", self.path)
    }
}

pub trait VfsNode: std::fmt::Debug {
    fn list(&self, path: &VfsQueryDirectory) -> VfsResult<VfsList>;
    fn read(&self, path: &VfsQueryFile) -> VfsResult<VfsRawData>;
    fn compare(&self, hash: &VfsHash) -> bool;
    fn hash(&self) -> &VfsHash;
    fn boxed(self) -> Box<dyn VfsNode>;
}

#[derive(Debug)]
pub struct VfsNodeEntry {
    pub node: Box<dyn VfsNode>,
    /// the path under wich the node will be mounted
    pub path: VfsPath,
}

#[derive(Default, Debug)]
pub struct Vfs {
    pub nodes: Vec<VfsNodeEntry>,
}

/// An abstraction of a filesystem where nodes can be inserted at specific locations
impl Vfs {
    // inserts a node into the vfs stack
    pub fn insert_node(&mut self, node: impl VfsNode, path: VfsPath) {
        let n = node.boxed();
        self.nodes.push(VfsNodeEntry { node: n, path });
    }
    pub fn remove_node(&mut self, node: impl VfsNode) -> VfsResult<()> {
        let node_hash = node.hash();
        let index = match self.nodes.iter().position(|x| x.node.hash() == node_hash) {
            Some(i) => i,
            None => return Err(VfsError::NodeNotFoundError),
        };
        self.nodes.remove(index);
        Ok(())
    }

    /// list all entries in a directory
    pub fn list(&self, directory: VfsQueryDirectory) -> VfsResult<Vec<VfsList>> {
        let mut entries = vec![];
        for n in &self.nodes {
            // if !directory.path.starts_with(&n.path) {
            if !n.path.starts_with(&directory.path) {
                continue;
            }

            // check if the we are still below the mount path
            let p_diff = n.path.subtract(&directory.path);
            if !p_diff.is_empty() {
                let entry = VfsEntry::Directory(VfsEntryDirectory {
                    path: VfsPath::new(&p_diff.nodes[0])?,
                    meta: VfsMetaData::default(),
                });
                let node_entry = VfsList {
                    node_hash: n.node.hash().clone(),
                    entries: vec![entry],
                };
                entries.push(node_entry);
                continue;
            }

            let p = directory.path.subtract(&n.path);

            let node_entries = n.node.list(&VfsQueryDirectory { path: p })?;
            entries.push(node_entries);
        }
        Ok(entries)
    }

    pub fn read(&self, file: VfsQueryFile, node_hash: Option<VfsHash>) -> VfsResult<VfsRawData> {
        for node in &self.nodes {
            let mut file_path = file.clone();
            file_path.path = file_path.path.subtract(&node.path);
            // if the node hash matches return the result
            if let Some(ref node_hash) = node_hash {
                if *node.node.hash() == *node_hash {
                    return node.node.read(&file_path);
                }
                // we dont need to do the other stuff
                continue;
            }

            if let Ok(r) = node.node.read(&file_path) {
                return Ok(r);
            };
        }
        if let Some(node_hash) = node_hash {
            Err(VfsError::NodeHashNotFoundError(node_hash))
        } else {
            Err(VfsError::FileNotFound(file.clone()))
        }
    }
}

impl Display for Vfs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = Vec::new();
        let c = self.nodes.len();
        let _ = writeln!(&mut buffer, "Vfs: nodes({})\n", c);
        for node in &self.nodes {
            let _ = writeln!(&mut buffer, "Node: {}\n", node.node.hash());
            let _ = writeln!(&mut buffer, "\tvfs path: {}\n", node.path);
        }
        let s = String::from_utf8(buffer).unwrap();
        write!(f, "{}", s)
    }
}
