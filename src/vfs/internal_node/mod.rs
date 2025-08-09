use std::{collections::HashMap, sync::Arc};

use crate::vfs::{Result, VfsEntry, VfsHash, VfsNode, VfsPath, VfsRawData};
use uuid::Uuid;

use super::{meta::VfsMetaData, VfsEntryFile, VfsQueryDirectory, VfsQueryFile};

mod directory;
mod file;
mod pak;

#[derive(Debug, Default, Clone)]
enum VfsInternalNodeType {
    #[default]
    None,
    File(file::File),
    Directory(directory::Directory),
    Pak(pak::PakAbstraction),
}

#[derive(Debug, Clone)]
pub struct VfsInternalNode {
    data: VfsInternalNodeType,
    hash: VfsHash,
}

#[derive(Debug, Default, Clone)]
pub struct VfsList {
    pub node_hash: VfsHash,
    pub entries: Vec<VfsEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VfsFlattenedListEntry {
    pub nodes: Vec<VfsHash>,
    pub entry: VfsEntry,
}

impl Into<VfsQueryFile> for VfsEntry {
    fn into(self) -> VfsQueryFile {
        VfsQueryFile { path: self.path() }
    }
}

#[allow(dead_code)]
impl VfsFlattenedListEntry {
    pub fn flatten_files(lists: Vec<VfsList>) -> Vec<VfsEntryFile> {
        let mut r: Vec<VfsEntryFile> = vec![];
        for list in &lists {
            for entry in &list.entries {
                match entry {
                    VfsEntry::File(vfs_entry_file) => r.push(vfs_entry_file.clone()),
                    VfsEntry::Directory(_) => {}
                };
            }
        }
        r
    }

    pub fn flatten(lists: Vec<VfsList>) -> Vec<VfsFlattenedListEntry> {
        let mut hash_map: HashMap<String, VfsFlattenedListEntry> = HashMap::new();
        for list in &lists {
            for entry in &list.entries {
                let key = entry.path_prefix();
                let value = list.node_hash.clone();
                hash_map
                    .entry(key.into())
                    .and_modify(|flattened_list| flattened_list.nodes.push(value.clone()))
                    .or_insert(VfsFlattenedListEntry {
                        nodes: vec![value],
                        entry: entry.clone(),
                    });
            }
        }
        hash_map.into_values().collect()
    }
}

#[allow(dead_code)]
impl VfsInternalNode {
    pub fn new_from_pak(pak: crate::pak::Pak, meta: VfsMetaData) -> Self {
        let mut files = vec![];
        for (index, f) in pak.files.iter().enumerate() {
            let mut m = meta.clone();
            m.size = f.size.into();
            files.push(pak::PakAbstractionFile {
                index,
                path: VfsPath::new(&f.name_as_string()).unwrap(),
                meta: m,
            })
        }
        let hash: String = format!("pak::{}", Uuid::new_v4());
        let pak_abstraction = pak::PakAbstraction { pak, files, meta };
        let data = VfsInternalNodeType::Pak(pak_abstraction);
        VfsInternalNode { data, hash }
    }
    pub fn new_from_file(file: std::path::PathBuf) -> Self {
        let data = VfsInternalNodeType::File(file);
        let hash: String = format!("file::{}", Uuid::new_v4());
        VfsInternalNode { data, hash }
    }
    pub fn new_from_directory(directory: std::path::PathBuf) -> Self {
        let data = VfsInternalNodeType::Directory(directory);
        let hash: String = format!("directory::{}", Uuid::new_v4());
        VfsInternalNode { data, hash }
    }
}

impl VfsNode for VfsInternalNode {
    fn list(&self, path: &VfsQueryDirectory) -> Result<VfsList> {
        match &self.data {
            VfsInternalNodeType::None => todo!(),
            VfsInternalNodeType::File(f) => file::list(f, path, self.hash()),
            VfsInternalNodeType::Directory(d) => directory::list(d, path, self.hash()),
            VfsInternalNodeType::Pak(pak) => pak::list(pak, path, self.hash()),
        }
    }

    fn read(&self, path: &VfsQueryFile) -> Result<VfsRawData> {
        match &self.data {
            VfsInternalNodeType::None => todo!(),
            VfsInternalNodeType::File(f) => file::read(f),
            VfsInternalNodeType::Directory(d) => directory::read(d, path, self.hash()),
            VfsInternalNodeType::Pak(pak) => pak::read(pak, path, self.hash()),
        }
    }

    fn exists(&self, path: &VfsQueryFile) -> bool {
        match &self.data {
            VfsInternalNodeType::None => todo!(),
            VfsInternalNodeType::File(f) => file::exists(f),
            VfsInternalNodeType::Directory(d) => directory::exists(d, path, self.hash()),
            VfsInternalNodeType::Pak(pak) => pak::exists(pak, path, self.hash()),
        }
    }

    fn compare(&self, hash: &VfsHash) -> bool {
        self.hash() == hash
    }

    fn hash(&self) -> &VfsHash {
        &self.hash
    }

    fn boxed(self) -> Arc<Box<dyn VfsNode>> {
        Arc::new(Box::new(self.clone()))
    }
}
