use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir_all, read, write};
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone)]
pub struct RealFileSystem {
    root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FakeFileSystem {
    root: PathBuf,
    mapping: Rc<RefCell<HashMap<PathBuf, Vec<u8>>>>,
}

pub trait FileSystem: Clone {
    fn subsystem<P: AsRef<Path>>(&self, path: P) -> Self;
    fn exists<P: AsRef<Path>>(&self, path: P) -> bool;
    fn read<P: AsRef<Path>>(&self, path: P) -> IoResult<Vec<u8>>;
    fn write<P: AsRef<Path>>(&mut self, path: P, contents: &[u8]) -> IoResult<()>;
    fn full_path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf;
}

impl FakeFileSystem {
    pub fn new() -> Self {
        FakeFileSystem {
            root: PathBuf::from("/"),
            mapping: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl FileSystem for RealFileSystem {
    fn subsystem<P: AsRef<Path>>(&self, path: P) -> Self {
        let mut new = self.clone();
        new.root.push(path);
        new
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        path.exists()
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> IoResult<Vec<u8>> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        read(path)
    }

    fn write<P: AsRef<Path>>(&mut self, path: P, contents: &[u8]) -> IoResult<()> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let mut path = self.root.join(path);
        path.pop();
        create_dir_all(&path)?;
        write(path, contents)
    }

    fn full_path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.join(path)
    }
}

impl FileSystem for FakeFileSystem {
    fn subsystem<P: AsRef<Path>>(&self, path: P) -> Self {
        let mut new = self.clone();
        new.root.push(path);
        new
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        self.mapping.borrow().contains_key(&path)
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> IoResult<Vec<u8>> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        match self.mapping.borrow().get(&path) {
            Some(contents) => Ok(contents.clone()),
            None => Err(IoError::new(
                ErrorKind::NotFound,
                format!("{:?} does not exist", path),
            )),
        }
    }

    fn write<P: AsRef<Path>>(&mut self, path: P, contents: &[u8]) -> IoResult<()> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        self.mapping.borrow_mut().insert(path, contents.to_owned());
        Ok(())
    }

    fn full_path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.join(path)
    }
}
