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

pub trait FileSystem {
    fn duplicate(&self) -> Box<FileSystem>;
    fn subsystem(&self, path: &Path) -> Box<FileSystem>;
    fn exists(&self, path: &Path) -> bool;
    fn read(&self, path: &Path, f: &mut FnMut(&mut Read) -> IoResult<()>) -> IoResult<()>;
    fn write(&self, path: &Path, f: &mut FnMut(&mut Write) -> IoResult<()>) -> IoResult<()>;
    fn full_path_for(&self, path: &Path) -> PathBuf;
    fn files(&self) -> Vec<PathBuf>;
}

#[cfg(test)]
impl FakeFileSystem {
    pub fn new() -> Self {
        FakeFileSystem {
            root: PathBuf::from("/"),
            mapping: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn is_empty(&self) -> bool {
        let root = self.root.clone();
        self.mapping
            .borrow()
            .keys()
            .filter(|p| p.starts_with(&root))
            .count()
            == 0
    }
}

impl FileSystem for RealFileSystem {
    fn subsystem(&self, path: &Path) -> Box<FileSystem> {
        assert!(path.is_relative(), "path must be relative");
        let mut new = self.clone();
        new.root.push(path);
        Box::new(new)
    }

    fn duplicate(&self) -> Box<FileSystem> {
        Box::new(self.clone())
    }

    fn exists(&self, path: &Path) -> bool {
        assert!(path.is_relative(), "path must be relative");
        let path = self.root.join(path);
        path.exists()
    }

    fn read(&self, path: &Path, f: &mut FnMut(&mut Read) -> IoResult<()>) -> IoResult<()> {
        assert!(path.is_relative(), "path must be relative");
        let path = self.root.join(path);
        match File::open(path) {
            Ok(mut file) => f(&mut file),
            Err(e) => Err(e),
        }
    }

    fn write(&self, path: &Path, f: &mut FnMut(&mut Write) -> IoResult<()>) -> IoResult<()> {
        assert!(path.is_relative(), "path must be relative");
        let path = self.root.join(path);
        create_dir_all(path.parent().unwrap())?;

        match File::create(path) {
            Ok(mut file) => f(&mut file),
            Err(e) => Err(e),
        }
    }

    fn full_path_for(&self, path: &Path) -> PathBuf {
        assert!(path.is_relative(), "path must be relative");
        self.root.join(path)
    }

    fn files(&self) -> Vec<PathBuf> {
        ::walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|p| p.path().to_owned())
            .filter_map(|p| p.strip_prefix(&self.root).ok().map(|p| p.to_owned()))
            .collect()
    }
}

#[cfg(test)]
impl FileSystem for FakeFileSystem {
    fn subsystem(&self, path: &Path) -> Box<FileSystem> {
        let mut new = self.clone();
        new.root.push(path);
        Box::new(new)
    }
    fn duplicate(&self) -> Box<FileSystem> {
        Box::new(self.clone())
    }

    fn exists(&self, path: &Path) -> bool {
        assert!(path.is_relative(), "path must be relative");
        let path = self.root.join(path);
        self.mapping.borrow().contains_key(&path)
    }

    fn read(&self, path: &Path, f: &mut FnMut(&mut Read) -> IoResult<()>) -> IoResult<()> {
        assert!(path.is_relative(), "path must be relative");
        let path = self.root.join(path);

        let contents = match self.mapping.borrow().get(&path) {
            Some(contents) => contents.clone(),
            None => {
                return Err(IoError::new(
                    ErrorKind::NotFound,
                    format!("{:?} does not exist", path),
                ))
            }
        };

        f(&mut &contents[..])
    }

    fn write(&self, path: &Path, f: &mut FnMut(&mut Write) -> IoResult<()>) -> IoResult<()> {
        assert!(path.is_relative(), "path must be relative");
        let path = self.root.join(path);

        let mut contents = vec![];
        f(&mut contents)?;

        self.mapping.borrow_mut().insert(path, contents);
        Ok(())
    }

    fn full_path_for(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    fn files(&self) -> Vec<PathBuf> {
        let root = self.root.clone();
        self.mapping
            .borrow()
            .keys()
            .filter_map(|p| p.strip_prefix(&root).ok())
            .map(|p| p.into())
            .collect()
    }
}
