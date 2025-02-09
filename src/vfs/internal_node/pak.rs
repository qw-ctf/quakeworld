use std::collections::HashMap;

use crate::{
    pak,
    vfs::{
        meta::VfsMetaData, path::VfsPath, VfsEntry, VfsEntryDirectory, VfsEntryFile, VfsError,
        VfsHash, VfsQueryDirectory, VfsQueryFile, VfsRawData, VfsResult,
    },
};

use super::VfsList;

#[derive(Debug, Default, Clone)]
pub struct PakAbstractionFile {
    pub index: usize,
    pub path: VfsPath,
    pub meta: VfsMetaData,
}

#[derive(Debug, Default, Clone)]
pub struct PakAbstraction {
    pub pak: pak::Pak,
    pub files: Vec<PakAbstractionFile>,
    pub meta: VfsMetaData,
}

// TODO: this seems to be far too complex for what it does
pub fn list(pak: &PakAbstraction, path: &VfsQueryDirectory, hash: &VfsHash) -> VfsResult<VfsList> {
    let mut entries = vec![];
    let mut found_directories: HashMap<String, usize> = HashMap::new();

    for f in &pak.files {
        if f.path.equals(&path.path) {
            let fe = VfsEntryFile {
                path: f.path.clone(),
                meta: f.meta.clone(),
                ..Default::default()
            };
            entries.push(VfsEntry::File(fe));
            continue;
        }
        if f.path.starts_with(&path.path) {
            let path_diff = f.path.diff(&path.path, 0);
            if path_diff.len() == 1 {
                let fe = VfsEntryFile {
                    path: path_diff,
                    meta: f.meta.clone(),
                    ..Default::default()
                };
                entries.push(VfsEntry::File(fe));
                continue;
            }
            let path_diff = f.path.diff(&path.path, 1);
            match found_directories.entry(path_diff.as_string()) {
                std::collections::hash_map::Entry::Occupied(_) => continue,
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(0);
                }
            };

            let de = VfsEntryDirectory {
                path: path_diff,
                meta: f.meta.clone(),
                ..Default::default()
            };
            entries.push(VfsEntry::Directory(de));
        }
    }
    Ok(VfsList {
        node_hash: hash.to_string(),
        entries,
    })
}

pub fn read(pak: &PakAbstraction, path: &VfsQueryFile, _hash: &VfsHash) -> VfsResult<VfsRawData> {
    for f in &pak.files {
        // println!(
        //     "we are here? -- ({}) ({}) {}",
        //     f.path.as_string(),
        //     path.path.as_string(),
        //     f.path.equals(&path.path)
        // );
        if f.path.equals(&path.path) {
            // FIXME: this seems stupid
            let d = pak.pak.get_data(&pak.pak.files[f.index])?;
            return Ok(d);
        }
    }
    Err(VfsError::FileNotFound(path.clone()))
}
