use std::error::Error;

pub fn vfs_mount(paks: Vec<String>) -> Result<quakeworld::vfs::Vfs, Box<dyn Error>> {
    let mut vfs = quakeworld::vfs::Vfs::new();
    for pak in paks {
        let data = super::read_file(pak.clone().into())?;
        let pp = quakeworld::pak::Pak::parse(pak.clone(), data, None)?;
        let node = quakeworld::vfs::VfsInternalNode::new_from_pak(
            pp,
            quakeworld::vfs::VfsMetaData::default(),
        );
        vfs.insert_node(node, "/");
    }
    Ok(vfs)
}

pub fn vfs_load_file(
    vfs: &quakeworld::vfs::Vfs,
    filename: impl Into<String>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    match vfs.read(filename.into(), None) {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn vfs_mount_load(
    paks: Vec<String>,
    filename: impl Into<String>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let vfs = vfs_mount(paks)?;
    vfs_load_file(&vfs, filename)
}
