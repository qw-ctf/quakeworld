use std::fmt::Display;
use std::io::Read;

use crate::vfs::path::VfsPath;
use crate::vfs::{
    VfsEntry, VfsEntryFile, VfsHash, VfsQueryDirectory, VfsQueryFile, VfsRawData, VfsResult,
};

use crate::vfs::VfsList;

// #[derive(Debug, Default, Clone)]
// pub struct File {}
pub type File = std::path::PathBuf;

pub fn read(file: &File, path: &VfsQueryFile) -> VfsResult<VfsRawData> {
    let mut f = std::fs::File::open(file)?;
    let metadata = std::fs::metadata(file)?;
    let mut buffer = vec![0; metadata.len() as usize];
    let _ = f.read(&mut buffer)?;
    Ok(buffer)
}

pub fn list(file: &File, path: &VfsQueryDirectory, hash: &VfsHash) -> VfsResult<VfsList> {
    let mut entries = vec![];

    if let Ok(metadata) = std::fs::metadata(file) {
        if let Some(file_name) = file.file_name() {
            if let Some(file_name) = file_name.to_str() {
                let fe = VfsEntryFile {
                    path: VfsPath::new(file_name)?,
                    meta: metadata.into(),
                };
                entries.push(VfsEntry::File(fe));
            }
        }
    }
    Ok(VfsList {
        node_hash: hash.to_string(),
        entries,
    })
}
