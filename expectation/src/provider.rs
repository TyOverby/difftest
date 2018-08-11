use std::io::{Read, Result as IoResult, Write};
use std::path::{Path, PathBuf};

use super::filesystem::FileSystem;

pub struct WriteRequester<F: FileSystem> {
    pub(crate) fs: F,
    pub(crate) files: Vec<PathBuf>,
}

impl<F: FileSystem> WriteRequester<F> {
    pub fn request<S, Fn>(&mut self, path: S, f: Fn) -> IoResult<()>
    where
        S: AsRef<Path>,
        Fn: for<'a> FnOnce(&'a mut Write) -> IoResult<()>,
    {
        self.files.push(self.fs.full_path_for(&path));
        self.fs.write(path, f)
    }
}

pub struct Provider<F: FileSystem> {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) root_fs: F,
    pub(crate) fs: F,
    pub(crate) files: Vec<(
        PathBuf,
        Box<for<'a> Fn(&'a mut Read, &'a mut Read) -> IoResult<bool>>,
        Box<
            for<'b> Fn(&'b mut Read, &'b mut Read, &'b Path, &'b mut WriteRequester<F>)
                -> IoResult<()>,
        >,
    )>,
}

struct Writer<F: FileSystem> {
    inner: Vec<u8>,
    filesystem: F,
    path: PathBuf,
}

impl<F: FileSystem> Provider<F> {
    pub fn new(root_fs: F, fs: F) -> Provider<F> {
        Provider {
            root_fs,
            fs,
            files: vec![],
        }
    }
}

impl<F: FileSystem> Write for Writer<F> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> IoResult<()> {
        self.inner.flush()
    }
}

impl<F: FileSystem> Drop for Writer<F> {
    fn drop(&mut self) {
        let mut contents = Vec::new();
        ::std::mem::swap(&mut contents, &mut self.inner);
        // TODO: maybe don't ignore?
        let _ = self.filesystem
            .write(&self.path, |w| w.write_all(&contents));
    }
}

impl<F: FileSystem> Provider<F> {
    pub fn custom_test<'x, S, C, D>(&'x mut self, name: S, compare: C, diff: D) -> impl Write + 'x
    where
        S: Into<PathBuf>,
        C: for<'a> Fn(&'a mut Read, &'a mut Read) -> IoResult<bool> + 'static,
        D: for<'b> Fn(&'b mut Read, &'b mut Read, &'b Path, &'b mut WriteRequester<F>)
                -> IoResult<()>
            + 'static,
    {
        let name = name.into();
        self.files
            .push((name.clone(), Box::new(compare), Box::new(diff)));
        Writer {
            inner: vec![],
            filesystem: self.fs.clone(),
            path: name,
        }
    }
}
