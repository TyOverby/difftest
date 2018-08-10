#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::io::{Error as IoError, ErrorKind};
#[cfg(test)]
use std::rc::Rc;

use std::fs::{create_dir_all, File};
use std::io::{Read, Result as IoResult, Write};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct RealFileSystem {
    pub root: PathBuf,
}

#[derive(Clone, Debug)]
#[cfg(test)]
pub struct FakeFileSystem {
    root: PathBuf,
    mapping: Rc<RefCell<HashMap<PathBuf, Vec<u8>>>>,
}

pub trait FileSystem: Clone {
    fn subsystem<P: AsRef<Path>>(&self, path: P) -> Self;
    fn exists<P: AsRef<Path>>(&self, path: P) -> bool;
    fn read<P: AsRef<Path>, F: FnOnce(&mut Read) -> IoResult<R>, R>(
        &self,
        path: P,
        f: F,
    ) -> IoResult<R>;
    fn write<P: AsRef<Path>, F: FnOnce(&mut Write) -> IoResult<()>>(
        &mut self,
        path: P,
        f: F,
    ) -> IoResult<()>;
    fn full_path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf;
}

#[cfg(test)]
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

    fn read<P: AsRef<Path>, F: FnOnce(&mut Read) -> IoResult<R>, R>(
        &self,
        path: P,
        f: F,
    ) -> IoResult<R> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        match File::open(path) {
            Ok(mut file) => f(&mut file),
            Err(e) => Err(e),
        }
    }

    fn write<P: AsRef<Path>, F: FnOnce(&mut Write) -> IoResult<()>>(
        &mut self,
        path: P,
        f: F,
    ) -> IoResult<()> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);
        create_dir_all(path.parent().unwrap())?;

        match File::create(path) {
            Ok(mut file) => f(&mut file),
            Err(e) => Err(e),
        }
    }

    fn full_path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.join(path)
    }
}

#[cfg(test)]
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

    fn read<P: AsRef<Path>, F: FnOnce(&mut Read) -> IoResult<R>, R>(
        &self,
        path: P,
        f: F,
    ) -> IoResult<R> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);

        match self.mapping.borrow().get(&path) {
            Some(contents) => f(&mut &contents[..]),
            None => Err(IoError::new(
                ErrorKind::NotFound,
                format!("{:?} does not exist", path),
            )),
        }
    }

    fn write<P: AsRef<Path>, F: FnOnce(&mut Write) -> IoResult<()>>(
        &mut self,
        path: P,
        f: F,
    ) -> IoResult<()> {
        assert!(path.as_ref().is_relative(), "path must be relative");
        let path = self.root.join(path);

        let mut contents = vec![];
        f(&mut contents)?;

        self.mapping.borrow_mut().insert(path, contents);
        Ok(())
    }

    fn full_path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.join(path)
    }
}
