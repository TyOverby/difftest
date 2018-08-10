use std::io::{Read, Result as IoResult, Write};
use std::path::PathBuf;

use super::filesystem::FileSystem;

/*

difftest("test-name", |provider| {
    let writer = provider.custom_test("foo.txt", |a, b| texteq, |a, b| textdiff);
});

*/

pub struct Provider<F: FileSystem> {
    fs: F,
    files: Vec<(
        PathBuf,
        Box<for<'a> Fn(&'a mut Read, &'a mut Read) -> bool>,
        Box<for<'b> Fn(&'b mut Read, &'b mut Read, &'b mut Write)>,
    )>,
}

struct Writer<F: FileSystem> {
    inner: Vec<u8>,
    filesystem: F,
    path: PathBuf,
}

impl<F: FileSystem> Provider<F> {
    pub fn new(fs: F) -> Provider<F> {
        Provider { fs, files: vec![] }
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
    fn custom_test<'x, S, C, D>(&'x mut self, name: S, compare: C, diff: D) -> impl Write + 'x
    where
        S: Into<PathBuf>,
        C: for<'a> Fn(&'a mut Read, &'a mut Read) -> bool + 'static,
        D: for<'b> Fn(&'b mut Read, &'b mut Read, &'b mut Write) + 'static,
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
