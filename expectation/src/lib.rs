extern crate expectation_shared;
extern crate serde_json;
extern crate walkdir;

#[cfg(feature = "text")]
extern crate diff;

pub mod extensions;
mod filesystem;
mod ipc;
mod provider;
#[cfg(test)]
mod test;

use expectation_shared::{Result as EResult, ResultKind};
use filesystem::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub use provider::Writer;

pub type WriteRequester = provider::WriteRequester<RealFileSystem>;

pub type Provider = provider::Provider<RealFileSystem>;
#[cfg(test)]
type FakeProvider = provider::Provider<FakeFileSystem>;

fn should_continue(name: &str) -> bool {
    match std::env::var("CARGO_EXPECT_FILTER") {
        Ok(v) => name.contains(&v),
        Err(_) => true,
    }
}

fn file_filter(file: &Path) -> bool {
    match std::env::var("CARGO_EXPECT_FILES") {
        Ok(v) => v
            .split(",")
            .any(|ending| file.to_str().map(|f| f.ends_with(ending)).unwrap_or(false)),
        Err(_) => true,
    }
}

pub fn expect<F: FnOnce(&mut Provider)>(name: &str, f: F) {
    if !should_continue(name) {
        return;
    }

    let top_fs = RealFileSystem {
        root: "./expectation-tests".into(),
    };
    let act_fs = top_fs.subsystem("actual").subsystem(name);
    let mut provider = Provider::new(top_fs.clone(), act_fs);
    f(&mut provider);

    let mut succeeded = true;
    let results = validate(name, top_fs, provider, file_filter);

    ipc::send(name, &results);

    for result in results {
        match result.kind {
            ResultKind::Ok => {}
            ResultKind::ActualNotFound(double) => {
                println!("\"Actual\" file not found");
                println!("  expected          {}", double.expected.to_string_lossy());
                println!("  actual (missing)  {}", double.actual.to_string_lossy());
                succeeded = false;
            }
            ResultKind::ExpectedNotFound(double) => {
                println!("\"Expected\" file not found");
                println!(
                    "  expected (missing)  {}",
                    double.expected.to_string_lossy()
                );
                println!("  actual              {}", double.actual.to_string_lossy());
                succeeded = false;
            }
            ResultKind::Difference(tripple) => {
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
) -> Vec<EResult> {
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
            out.push(EResult::actual_not_found(
                name,
                &file,
                actual_fs.full_path_for(&file),
                expected_fs.full_path_for(&file),
            ));
            continue;
        }

        if !expected_fs.exists(&file) {
            out.push(EResult::expected_not_found(
                name,
                &file,
                actual_fs.full_path_for(&file),
                expected_fs.full_path_for(&file),
            ));
            continue;
        }

        let is_eq = actual_fs.read(&file, |actual_read| {
            expected_fs.read(&file, |expected_read| eq(actual_read, expected_read))
        });

        let is_eq = match is_eq {
            Ok(r) => r,
            Err(e) => {
                out.push(EResult::io_error(name, &file, e));
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

            out.push(EResult::difference(
                name,
                &file,
                actual_fs.full_path_for(&file),
                expected_fs.full_path_for(&file),
                write_requester.files,
            ));

            if let Err(e) = diff_result {
                out.push(EResult::io_error(name, &file, e));
            }
        }
    }

    for file in expected_fs.files() {
        if !filter(&file) || visited.contains(&file) {
            continue;
        }

        if !actual_fs.exists(&file) {
            out.push(EResult::actual_not_found(
                name,
                &file,
                actual_fs.full_path_for(&file),
                expected_fs.full_path_for(&file),
            ));
            continue;
        }
    }

    out
}
