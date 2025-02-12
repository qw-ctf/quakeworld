use std::{fmt::Display, rc::Rc};

/// Internal Path Definitions
use crate::vfs::Result;

#[derive(Clone, Debug, Default)]
pub struct VfsPath {
    pub nodes: Vec<Rc<str>>,
}

impl Display for VfsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.nodes.join("/");
        write!(f, "{}", s)
    }
}

impl From<&str> for VfsPath {
    fn from(value: &str) -> Self {
        match VfsPath::new(value) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        }
    }
}

impl From<&VfsPath> for String {
    fn from(val: &VfsPath) -> Self {
        val.to_string().clone()
    }
}

impl From<VfsPath> for String {
    fn from(val: VfsPath) -> Self {
        val.to_string().clone()
    }
}

impl VfsPath {
    pub fn new(path: &str) -> Result<VfsPath> {
        let mut nodes = vec![];
        let p = std::path::Path::new(path);
        for component in p.components() {
            match component {
                std::path::Component::Prefix(_) => todo!(),
                std::path::Component::RootDir => {}
                std::path::Component::CurDir => todo!(),
                std::path::Component::ParentDir => todo!(),
                std::path::Component::Normal(n) => {
                    if let Some(p) = n.to_str() {
                        nodes.push(Rc::from(p));
                    }
                }
            };
        }
        Ok(VfsPath { nodes })
    }

    pub fn push(&mut self, entry: impl Into<String>) {
        let entry = entry.into();
        self.nodes.push(entry.into());
    }

    pub fn subtract(&self, path: &VfsPath) -> VfsPath {
        let mut new_path = VfsPath {
            ..Default::default()
        };

        let mut path_iter = path.nodes.iter();
        for node in &self.nodes {
            if let Some(p) = path_iter.next() {
                if node == p {
                    continue;
                } else {
                    panic!("should probably handle this")
                }
            }
            new_path.push(node.to_string());
        }
        new_path
    }

    pub fn starts_with(&self, path: &VfsPath) -> bool {
        let mut path_iter = path.nodes.iter();
        if self.nodes.is_empty() {
            return path.nodes.is_empty();
        }
        for node in &self.nodes {
            let path_node = match path_iter.next() {
                Some(p) => p,
                None => return true,
            };
            if node != path_node {
                return false;
            }
        }
        true
    }

    pub fn extend(&mut self, path: &VfsPath) {
        for p in &path.nodes {
            self.nodes.push(Rc::clone(p));
        }
    }

    pub fn as_string(&self) -> String {
        self.nodes.join("/")
    }

    pub fn equals(&self, path: &VfsPath) -> bool {
        if self.nodes.len() != path.nodes.len() {
            return false;
        }
        let count = self
            .nodes
            .iter()
            .zip(&path.nodes)
            .filter(|&(a, b)| a != b)
            .count();

        count == 0
    }

    pub fn equals_string(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        self.to_string() == path
    }

    pub fn diff(&self, path: &VfsPath, depth: usize) -> VfsPath {
        let mut return_path = self.subtract(path);
        if depth == 0 {
            return return_path;
        }
        return_path.nodes.drain(depth..);
        return_path
    }

    pub fn pop(&mut self) {
        self.nodes.pop();
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::vfs::path::VfsPath;
    #[test]
    pub fn starts_with() -> Result<(), crate::vfs::Error> {
        // first path longer
        let p = VfsPath::new("test/problem")?;
        let p1 = VfsPath::new("test/")?;
        assert_eq!(true, p.starts_with(&p1));

        // both paths equal
        let p = VfsPath::new("test/")?;
        let p1 = VfsPath::new("test/")?;
        assert_eq!(true, p.starts_with(&p1));

        // first path longer than second
        let p = VfsPath::new("test/lala")?;
        let p1 = VfsPath::new("test/")?;
        assert_eq!(true, p.starts_with(&p1));

        // does not start with
        let p = VfsPath::new("test/lala")?;
        let p1 = VfsPath::new("fail/")?;
        assert_eq!(false, p.starts_with(&p1));

        // two empty paths
        let p = VfsPath::new("")?;
        let p1 = VfsPath::new("")?;
        assert_eq!(true, p.starts_with(&p1));

        // first path empty
        let p = VfsPath::new("")?;
        let p1 = VfsPath::new("test")?;
        assert_eq!(false, p.starts_with(&p1));

        Ok(())
    }

    #[test]
    pub fn subtract() -> Result<(), crate::vfs::Error> {
        let p = VfsPath::new("test/problem")?;
        let p1 = VfsPath::new("test/")?;
        let p_new = p.subtract(&p1);
        assert_eq!(p_new.as_string(), "problem");
        Ok(())
    }

    #[test]
    pub fn as_string() -> Result<(), crate::vfs::Error> {
        let p = VfsPath::new("/test/problem///")?;
        assert_eq!(p.as_string(), "test/problem");
        Ok(())
    }

    #[test]
    pub fn extend() -> Result<(), crate::vfs::Error> {
        let mut p = VfsPath::new("/test/")?;
        let p1 = VfsPath::new("/problem/solves")?;
        p.extend(&p1);
        assert_eq!(p.as_string(), "test/problem/solves");
        Ok(())
    }

    #[test]
    pub fn push() -> Result<(), crate::vfs::Error> {
        let mut p = VfsPath::new("/test/")?;
        p.push("problem".to_string());
        assert_eq!(p.as_string(), "test/problem");
        Ok(())
    }

    #[test]
    pub fn new() -> Result<(), crate::vfs::Error> {
        let p = VfsPath::new("//test//problem")?;
        assert_eq!(p.as_string(), "test/problem");
        Ok(())
    }

    #[test]
    pub fn diff() -> Result<(), crate::vfs::Error> {
        let p = VfsPath::new("/test/expected")?;
        let p1 = VfsPath::new("/test/")?;
        let p_diff = p.diff(&p1, 0);
        assert_eq!(p_diff.as_string(), "expected");

        let p = VfsPath::new("/test/expected/expected")?;
        let p1 = VfsPath::new("/test/")?;
        let p_diff = p.diff(&p1, 0);
        assert_eq!(p_diff.as_string(), "expected/expected");

        let p = VfsPath::new("/test/expected/error")?;
        let p1 = VfsPath::new("/test/")?;
        let p_diff = p.diff(&p1, 1);
        assert_eq!(p_diff.as_string(), "expected");
        Ok(())
    }

    #[test]
    pub fn equals() -> Result<(), crate::vfs::Error> {
        let p = VfsPath::new("/test/error")?;
        let p1 = VfsPath::new("/test/")?;
        assert_eq!(p.equals(&p1), false);

        let p = VfsPath::new("/test/")?;
        let p1 = VfsPath::new("/test/")?;
        assert_eq!(p.equals(&p1), true);
        Ok(())
    }

    #[test]
    pub fn equals_string() -> Result<(), crate::vfs::Error> {
        let path_str = "test/path";
        let path: VfsPath = path_str.try_into()?;
        let path_compare: VfsPath = path_str.try_into()?;

        assert!(path.equals_string(path_str));
        assert!(path.equals_string(path_compare));

        Ok(())
    }
}
