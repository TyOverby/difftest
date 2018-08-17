use std::io::{Read, Result as IoResult, Write};
use std::path::{Path, PathBuf};

use expectation_shared::filesystem::FileSystem;

pub struct WriteRequester {
    pub(crate) fs: Box<FileSystem>,
    pub(crate) files: Vec<PathBuf>,
}

impl WriteRequester {
    pub fn request<S, Fn>(&mut self, path: S, mut f: Fn) -> IoResult<()>
    where
        S: AsRef<Path>,
        Fn: for<'a> FnMut(&'a mut Write) -> IoResult<()>,
    {
        let mut v = vec![];
        v.push(1u8);
        self.files.push(self.fs.full_path_for(path.as_ref()));
        self.fs.write(path.as_ref(), &mut f)
    }
}

pub struct Provider {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) root_fs: Box<FileSystem>,
    pub(crate) fs: Box<FileSystem>,
    pub(crate) files: Vec<(
        PathBuf,
        Box<for<'a> Fn(&'a mut Read, &'a mut Read) -> IoResult<bool>>,
        Box<
            for<'b> Fn(&'b mut Read, &'b mut Read, &'b Path, &'b mut WriteRequester)
                -> IoResult<()>,
        >,
    )>,
}

pub struct Writer {
    inner: Vec<u8>,
    filesystem: Box<FileSystem>,
    path: PathBuf,
    written_to: bool,
}

impl Writer {
    fn new(filesystem: Box<FileSystem>, path: PathBuf) -> Self {
        Writer {
            filesystem,
            path,
            inner: vec![],
            written_to: false,
        }
    }
}

impl Provider {
    pub(crate) fn new(root_fs: Box<FileSystem>, fs: Box<FileSystem>) -> Provider {
        Provider {
            root_fs,
            fs,
            files: vec![],
        }
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.written_to = true;
        self.inner.write(buf)
    }
    fn flush(&mut self) -> IoResult<()> {
        self.inner.flush()
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if !self.written_to {
            return;
        }

        let mut contents = Vec::new();
        ::std::mem::swap(&mut contents, &mut self.inner);
        // TODO: maybe don't ignore?
        let _ = self
            .filesystem
            .write(&self.path, &mut |w| w.write_all(&contents));
    }
}

impl Provider {
    pub fn custom_test<S, C, D>(&mut self, name: S, compare: C, diff: D) -> Writer
    where
        S: AsRef<Path>,
        C: for<'a> Fn(&'a mut Read, &'a mut Read) -> IoResult<bool> + 'static,
        D: for<'b> Fn(&'b mut Read, &'b mut Read, &'b Path, &'b mut WriteRequester) -> IoResult<()>
            + 'static,
    {
        let name: PathBuf = name.as_ref().into();
        self.files
            .push((name.clone(), Box::new(compare), Box::new(diff)));
        Writer::new(self.fs.duplicate(), name )
    }
}

#[test]
fn writer_does_not_write_to_filesystem_if_not_written_to() {
    use expectation_shared::filesystem::*;
    let filesystem = Box::new(FakeFileSystem::new()) as Box<FileSystem>;
    {
        let _writer = Writer::new(filesystem.duplicate(), "foo.txt".into());
    }
    assert!(!filesystem.exists(Path::new("foo.txt")));
}
