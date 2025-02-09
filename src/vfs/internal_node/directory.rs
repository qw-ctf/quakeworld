use std::{
    fs::{self, File},
    io::Read,
};

use crate::vfs::{
    VfsEntry, VfsEntryDirectory, VfsEntryFile, VfsHash, VfsQueryDirectory, VfsQueryFile,
    VfsRawData, VfsResult,
};

use super::VfsList;

pub type Directory = std::path::PathBuf;

pub fn list(directory: &Directory, path: &VfsQueryDirectory, hash: &VfsHash) -> VfsResult<VfsList> {
    // println!("++++++++++\n{:?}\n{:?}\n----------", directory, path);
    let mut return_entries = vec![];
    let mut d = directory.clone();
    for p in &path.path.nodes {
        let s: &str = &*p;
        d.push(std::path::PathBuf::from(s));
    }

    if let Ok(entries) = std::fs::read_dir(d) {
        for entry in entries {
            let e = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let meta = std::fs::metadata(e.path())?;
            let name = e.file_name();
            let name = name.to_string_lossy();
            let mut p = path.path.clone();
            p.pop();
            p.push(name);

            if meta.is_dir() {
                let de = VfsEntryDirectory {
                    path: p,
                    meta: meta.into(),
                    ..Default::default()
                };
                return_entries.push(VfsEntry::Directory(de));
            } else if meta.is_file() {
                let fe = VfsEntryFile {
                    path: p,
                    meta: meta.into(),
                    ..Default::default()
                };
                return_entries.push(VfsEntry::File(fe));
            }
        }
    }
    Ok(VfsList {
        node_hash: hash.to_string(),
        entries: return_entries,
    })
}

pub fn read(file: &Directory, path: &VfsQueryFile, _hash: &VfsHash) -> VfsResult<VfsRawData> {
    let mut filename = file.clone();
    for p in &path.path.nodes {
        let s: &str = &*p;
        filename.push(std::path::PathBuf::from(s));
    }
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;
    Ok(buffer)
}
