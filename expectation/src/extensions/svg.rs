use super::super::provider::{Provider, WriteRequester};
use super::super::*;
use super::escape_html;

use super::text::{text_diff, text_eq};
use diff;
use std::fmt::Debug;
use std::io::{Read, Result as IoResult, Write};
use std::path::Path;

pub trait SvgDiffExtension {
    fn svg_writer<N>(&self, filename: N) -> Writer
    where
        N: AsRef<Path>;

    fn svg<N, S>(&self, filename: N, text: S) -> IoResult<()>
    where
        N: AsRef<Path>,
        S: AsRef<str>,
    {
        let mut w = self.svg_writer(filename);
        write!(w, "{}", text.as_ref())
    }
}

impl SvgDiffExtension for Provider {
    fn svg_writer<S>(&self, filename: S) -> Writer
    where
        S: AsRef<Path>,
    {
        self.custom_test(
            filename,
            |a, b| text_eq(a, b),
            |a, b, c, d| svg_diff(a, b, c, d),
        )
    }
}

pub(crate) fn svg_diff<R1: Read, R2: Read>(
    r1: R1,
    r2: R2,
    path: &Path,
    write_requester: &mut WriteRequester,
) -> IoResult<()> {
    text_diff(r1, r2, path, write_requester)?;
    write_requester.set_html_renderer(|actual, expected, _| {
        return format!(
            r#"
        <h3> Actual </h3>
        <img src="{}"/>
        <h3> Expected </h3>
        <img src="{}"/>

        "#,
            actual.to_string_lossy(),
            expected.to_string_lossy(),
        );
    });
    Ok(())
}
