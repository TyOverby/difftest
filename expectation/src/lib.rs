extern crate walkdir;

mod filesystem;
mod provider;
#[cfg(test)]
mod test;

use filesystem::*;
use std::collections::HashSet;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

pub type WriteRequester = provider::WriteRequester<RealFileSystem>;

pub type Provider = provider::Provider<RealFileSystem>;
#[cfg(test)]
type FakeProvider = provider::Provider<FakeFileSystem>;

#[derive(Debug, PartialEq)]
pub struct Double {
    actual: PathBuf,
    expected: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct Tripple {
    actual: PathBuf,
    expected: PathBuf,
    diffs: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum Result {
    Ok(Double),
    ExpectedNotFound(Double),
    ActualNotFound(Double),
    Difference(Tripple),
    IoError(IoError),
}

impl PartialEq for Result {
    fn eq(&self, other: &Self) -> bool {
        use Result::*;
        match (self, other) {
            (Ok(a), Ok(b)) => a == b,
            (ExpectedNotFound(a), ExpectedNotFound(b)) => a == b,
            (ActualNotFound(a), ActualNotFound(b)) => a == b,
            (Difference(a), Difference(b)) => a == b,
            (IoError(_), IoError(_)) => false,
            _ => false,
        }
    }
}

pub fn difftest<F: FnOnce(&mut Provider)>(name: &str, f: F) {
    let top_fs = RealFileSystem { root: "f".into() };
    let act_fs = top_fs.subsystem("actual").subsystem(name);
    let mut provider = Provider::new(top_fs, act_fs);
    f(&mut provider);
}

pub fn validate<F: FileSystem + 'static, Fi: Fn(&Path) -> bool>(
    name: &str,
    fs: F,
    provider: provider::Provider<F>,
    filter: Fi,
) -> Vec<Result> {
    let mut visited = HashSet::new();
    let mut out = Vec::new();

    let expected_fs = fs.subsystem("expected").subsystem(name);
    let actual_fs = fs.subsystem("actual").subsystem(name);
    let diff_fs = fs.subsystem("diff").subsystem(name);

    #[allow(unused_variables)]
    let fs = ();

    for (file, eq, diff) in provider.files {
        if !filter(&file) || visited.contains(&file) {
            continue;
        }
        visited.insert(file.clone());

        if !actual_fs.exists(&file) {
            out.push(Result::ActualNotFound(Double {
                actual: actual_fs.full_path_for(&file),
                expected: expected_fs.full_path_for(&file),
            }));
            continue;
        }

        if !expected_fs.exists(&file) {
            out.push(Result::ExpectedNotFound(Double {
                actual: actual_fs.full_path_for(&file),
                expected: expected_fs.full_path_for(&file),
            }));
            continue;
        }

        let is_eq = actual_fs.read(&file, |actual_read| {
            expected_fs.read(&file, |expected_read| eq(actual_read, expected_read))
        });

        let is_eq = match is_eq {
            Ok(r) => r,
            Err(e) => {
                out.push(Result::IoError(e));
                continue;
            }
        };

        if !is_eq {
            let mut write_requester = provider::WriteRequester {
                fs: diff_fs.clone(),
                files: vec![],
            };

            let diff_result = actual_fs.read(&file, |actual_read| {
                expected_fs.read(&file, |expected_read| {
                    diff(actual_read, expected_read, &file, &mut write_requester)
                })
            });

            out.push(Result::Difference(Tripple {
                actual: actual_fs.full_path_for(&file),
                expected: expected_fs.full_path_for(&file),
                diffs: write_requester.files,
            }));

            if let Err(e) = diff_result {
                out.push(Result::IoError(e));
            }
        }
    }

    for file in expected_fs.files() {
        if !filter(&file) || visited.contains(&file) {
            continue;
        }

        if !actual_fs.exists(&file) {
            out.push(Result::ActualNotFound(Double {
                actual: actual_fs.full_path_for(&file),
                expected: expected_fs.full_path_for(&file),
            }));
            continue;
        }
    }

    out
}
