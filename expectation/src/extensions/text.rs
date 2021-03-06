use super::super::provider::{Provider, WriteRequester};
use super::super::*;
use super::escape_html;

use diff;
use std::fmt::Debug;
use std::io::{Read, Result as IoResult, Write};
use std::path::Path;

pub trait TextDiffExtension {
    fn text_writer<N>(&self, filename: N) -> Writer
    where
        N: AsRef<Path>;

    fn text<N, S>(&self, filename: N, text: S) -> IoResult<()>
    where
        N: AsRef<Path>,
        S: AsRef<str>,
    {
        let mut w = self.text_writer(filename);
        write!(w, "{}", text.as_ref())
    }

    fn debug<N, D>(&self, filename: N, object: D) -> IoResult<()>
    where
        N: AsRef<Path>,
        D: Debug,
    {
        let mut w = self.text_writer(filename);
        write!(w, "{:#?}", object)
    }
}

impl TextDiffExtension for Provider {
    fn text_writer<S>(&self, filename: S) -> Writer
    where
        S: AsRef<Path>,
    {
        self.custom_test(
            filename,
            |a, b| text_eq(a, b),
            |a, b, c, d| text_diff(a, b, c, d),
        )
    }
}

pub(crate) fn text_eq<R1: Read, R2: Read>(mut r1: R1, mut r2: R2) -> IoResult<bool> {
    let mut v1 = Vec::new();
    let mut v2 = Vec::new();
    r1.read_to_end(&mut v1)?;
    r2.read_to_end(&mut v2)?;

    Ok(v1 == v2)
}

fn add_extension(p: &Path, new_ext: &str) -> PathBuf {
    let old_ext = match p.extension() {
        Some(e) => e.to_string_lossy().into_owned(),
        None => "".to_owned(),
    };
    p.with_extension(format!("{}{}", old_ext, new_ext))
}

pub(crate) fn text_diff<R1: Read, R2: Read>(
    mut r1: R1,
    mut r2: R2,
    path: &Path,
    write_requester: &mut WriteRequester,
) -> IoResult<()> {
    let mut s1 = String::new();
    let mut s2 = String::new();
    let mut diff = Vec::new();
    r1.read_to_string(&mut s1)?;
    r2.read_to_string(&mut s2)?;

    for d in diff::lines(&s1, &s2) {
        match d {
            diff::Result::Left(l) => writeln!(diff, "+{}", l)?,
            diff::Result::Both(l, _) => writeln!(diff, " {}", l)?,
            diff::Result::Right(r) => writeln!(diff, "-{}", r)?,
        }
    }

    let diff = String::from_utf8(diff).unwrap();

    write_requester.request(add_extension(path, ".diff"), |w| write!(w, "{}", diff))?;

    write_requester.set_html_renderer(move |_, _, _| {
        let mut html = Vec::new();

        write!(html, "<h3> Actual </h3>");
        write!(html, "<code><pre>{}</pre></code>", escape_html(&s1));

        write!(html, "<h3> Expected </h3>");
        write!(html, "<code><pre>{}</pre></code>", escape_html(&s2));

        write!(html, "<h3> Diff </h3>");
        write!(html, "<code><pre>{}</pre></code>", escape_html(&diff));

        String::from_utf8(html).unwrap()
    });

    Ok(())
}
