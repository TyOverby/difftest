extern crate walkdir;

#[cfg(feature = "text")]
extern crate diff;

pub mod extensions;
mod filesystem;
mod provider;
#[cfg(test)]
mod test;

use filesystem::*;
use std::collections::HashSet;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

pub use provider::Writer;

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

pub fn expect<F: FnOnce(&mut Provider)>(name: &str, f: F) {
    let top_fs = RealFileSystem {
        root: "./expectation-tests".into(),
    };
    let act_fs = top_fs.subsystem("actual").subsystem(name);
    let mut provider = Provider::new(top_fs.clone(), act_fs);
    f(&mut provider);

    let mut succeeded = true;
    let results = validate(name, top_fs, provider, |_| true);
    for result in results {
        match result {
            Result::Ok(_) => {}
            Result::ActualNotFound(double) => {
                println!("\"Actual\" file not found");
                println!("  expected          {}", double.expected.to_string_lossy());
                println!("  actual (missing)  {}", double.actual.to_string_lossy());
                succeeded = false;
            }
            Result::ExpectedNotFound(double) => {
                println!("\"Expected\" file not found");
                println!(
                    "  expected (missing)  {}",
                    double.expected.to_string_lossy()
                );
                println!("  actual              {}", double.actual.to_string_lossy());
                succeeded = false;
            }
            Result::Difference(tripple) => {
                println!("Files differ");
                println!("  expected  {}", tripple.expected.to_string_lossy());
                println!("  actual    {}", tripple.actual.to_string_lossy());
                match tripple.diffs.len() {
                    0 => {}
                    1 => println!("  diff      {}", tripple.diffs[0].to_string_lossy()),
                    _ => {
                        println!("  diffs");
                        for diff in tripple.diffs {
                            println!("    {}", diff.to_string_lossy());
                        }
                    }
                }
                succeeded = false;
            }
            _ => {}
        }
    }
    if !succeeded {
        panic!("Expectation test found some errors.");
    }
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
