extern crate expectation_shared;
extern crate serde_json;

#[cfg(feature = "text")]
extern crate diff;

pub mod extensions;
mod ipc;
mod provider;
#[cfg(test)]
mod test;

pub use provider::Provider;

use expectation_shared::{Result as EResult, ResultKind};
use expectation_shared::filesystem::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub use provider::Writer;

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
    if !name.starts_with("expectation_test_") {
        panic!("expectation test {} is an invalid test name.  It must start with \"expectation_test_\"", name);
    }

    let name = name.trim_left_matches("expectation_test_");
    if !should_continue(name) {
        return;
    }

    let top_fs = RealFileSystem {
        root: Path::new("./expectation-tests").canonicalize().unwrap()
    };
    let act_fs = top_fs
        .subsystem(Path::new("actual"))
        .subsystem(Path::new(name));
    let mut provider = Provider::new(top_fs.duplicate(), act_fs.duplicate());
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

#[macro_export]
macro_rules! expectation_test {
    (fn $name:ident ($provider:ident : $type:ty) $body:tt) => {
        #[test]
        fn $name() {
            $crate::expect(stringify!($name), (|$provider: $type| $body));
        }
    };
}

fn validate<F: FileSystem + 'static, Fi: Fn(&Path) -> bool>(
    name: &str,
    fs: F,
    provider: Provider,
    filter: Fi,
) -> Vec<EResult> {
    let mut visited = HashSet::new();
    let mut out = Vec::new();

    let expected_fs = fs
        .subsystem(Path::new("expected"))
        .subsystem(Path::new(name));
    let actual_fs = fs.subsystem(Path::new("actual")).subsystem(Path::new(name));
    let diff_fs = fs.subsystem(Path::new("diff")).subsystem(Path::new(name));

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

        let mut is_eq = false;
        let res = actual_fs.read(&file, &mut |actual_read| {
            expected_fs.read(&file, &mut |expected_read| {
                is_eq = eq(actual_read, expected_read)?;
                Ok(())
            })
        });

        let is_eq = match res {
            Ok(_) => is_eq,
            Err(e) => {
                out.push(EResult::io_error(name, &file, e));
                continue;
            }
        };

        if !is_eq {
            let mut write_requester = provider::WriteRequester {
                fs: diff_fs.duplicate(),
                files: vec![],
            };

            let diff_result = actual_fs.read(&file, &mut |actual_read| {
                expected_fs.read(&file, &mut |expected_read| {
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
            continue;
        }

        out.push(EResult::ok(name, &file));
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
