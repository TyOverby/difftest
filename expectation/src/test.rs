use super::*;
use super::extensions::*;
use std::io::{Read, Result as IoResult};

fn byte_for_byte_equality<R1: Read, R2: Read>(mut r1: R1, mut r2: R2) -> IoResult<bool> {
    let mut v1 = vec![];
    let mut v2 = vec![];

    r1.read_to_end(&mut v1)?;
    r2.read_to_end(&mut v2)?;

    Ok(v1 == v2)
}

fn byte_for_byte_diff<R1: Read, R2: Read>(
    mut r1: R1,
    mut r2: R2,
    name: &Path,
    w: &mut provider::WriteRequester<filesystem::FakeFileSystem>,
) -> IoResult<()> {
    let mut v1 = vec![];
    let mut v2 = vec![];

    r1.read_to_end(&mut v1)?;
    r2.read_to_end(&mut v2)?;

    let diff_name = match name.file_name().and_then(|a| a.to_str()) {
        Some(name) => format!("{}.diff", name),
        None => "diff".into(),
    };

    w.request(diff_name, |w| {
        writeln!(w, "actual: ")?;
        writeln!(w, "{:?}", v1)?;
        writeln!(w, "expected: ")?;
        writeln!(w, "{:?}", v2)?;
        Ok(())
    })
}

#[cfg(test)]
pub fn difftest_prepare<F: FnOnce(&mut FakeProvider)>(name: &str, f: F) -> FakeFileSystem {
    let top_fs = filesystem::FakeFileSystem::new();
    let mut provider =
        provider::Provider::new(top_fs.clone(), top_fs.subsystem("actual").subsystem(name));
    f(&mut provider);
    top_fs
}

#[cfg(test)]
pub fn difftest_validate<F: FnOnce(&mut FakeProvider)>(name: &str, f: F) -> Vec<Result> {
    let top_fs = filesystem::FakeFileSystem::new();
    let mut provider =
        provider::Provider::new(top_fs.clone(), top_fs.subsystem("actual").subsystem(name));
    f(&mut provider);
    validate(name, top_fs, provider, |_| true)
}

#[test]
fn not_used_provider() {
    let fs = difftest_prepare("hi", |_provider| {});
    assert!(fs.is_empty());
}

#[test]
fn provider_used_once() {
    use std::io::Write;

    let fs = difftest_prepare("hi", |provider| {
        let mut w = provider.custom_test(
            "foo.txt",
            |_, _| unimplemented!(),
            |_, _, _, _| unimplemented!(),
        );
        write!(w, "hello world").unwrap();
    });

    println!("{:#?}", fs);

    assert!(!fs.is_empty());
    fs.read("actual/hi/foo.txt", |r| {
        let mut v = String::new();
        r.read_to_string(&mut v)?;
        assert_eq!(v, "hello world");
        Ok(())
    }).unwrap();
}

#[test]
fn provider_used_more_than_once() {
    use std::io::Write;

    let fs = difftest_prepare("hi", |provider| {
        {
            let mut w = provider.custom_test(
                "foo.txt",
                |_, _| unimplemented!(),
                |_, _, _, _| unimplemented!(),
            );
            write!(w, "hello world").unwrap();
        }
        {
            let mut w = provider.custom_test(
                "bar.txt",
                |_, _| unimplemented!(),
                |_, _, _, _| unimplemented!(),
            );
            write!(w, "hello world").unwrap();
        }
    });

    assert!(!fs.is_empty());
    fs.read("actual/hi/foo.txt", |r| {
        let mut v = String::new();
        r.read_to_string(&mut v)?;
        assert_eq!(v, "hello world");
        Ok(())
    }).unwrap();

    fs.read("actual/hi/bar.txt", |r| {
        let mut v = String::new();
        r.read_to_string(&mut v)?;
        assert_eq!(v, "hello world");
        Ok(())
    }).unwrap();
}

#[test]
fn validate_on_no_files() {
    let results = difftest_validate("hi", |_provider| {});
    assert!(results.is_empty());
}

#[test]
fn validate_one_file_expected_not_found() {
    use std::io::Write;
    let results = difftest_validate("hi", |provider| {
        let mut w = provider.custom_test(
            "foo.txt",
            |_, _| unimplemented!(),
            |_, _, _, _| unimplemented!(),
        );
        write!(w, "hello world").unwrap();
    });

    assert_eq!(
        results,
        vec![Result::ExpectedNotFound(Double {
            actual: "/actual/hi/foo.txt".into(),
            expected: "/expected/hi/foo.txt".into(),
        })]
    );
}

#[test]
fn validate_one_file_actual_not_found() {
    use std::io::Write;
    let results = difftest_validate("hi", |provider| {
        provider
            .root_fs
            .write("expected/hi/something_else.txt", |writer| {
                write!(writer, "not found")
            }).unwrap();

        let mut w = provider.custom_test(
            "foo.txt",
            |_, _| unimplemented!(),
            |_, _, _, _| unimplemented!(),
        );
        write!(w, "hello world").unwrap();
    });

    assert_eq!(
        results,
        vec![
            Result::ExpectedNotFound(Double {
                actual: "/actual/hi/foo.txt".into(),
                expected: "/expected/hi/foo.txt".into(),
            }),
            Result::ActualNotFound(Double {
                actual: "/actual/hi/something_else.txt".into(),
                expected: "/expected/hi/something_else.txt".into(),
            }),
        ]
    );
}

#[test]
fn validate_one_file_diff_is_bad() {
    use std::io::Write;
    let results = difftest_validate("hi", |provider| {
        provider
            .root_fs
            .write("expected/hi/foo.txt", |writer| {
                write!(writer, "goodbye found")
            }).unwrap();

        let mut w = provider.custom_test(
            "foo.txt",
            |a, b| byte_for_byte_equality(a, b),
            |a, b, n, w| byte_for_byte_diff(a, b, n, w),
        );
        write!(w, "hello world").unwrap();
    });

    assert_eq!(
        results,
        vec![Result::Difference(Tripple {
            actual: "/actual/hi/foo.txt".into(),
            expected: "/expected/hi/foo.txt".into(),
            diffs: vec!["/diff/hi/foo.txt.diff".into()],
        })]
    );
}

#[test]
fn validate_one_file_diff_is_bad_with_text_extension() {
    use std::io::Write;
    let results = difftest_validate("hi", |provider| {
        provider
            .root_fs
            .write("expected/hi/foo.txt", |writer| {
                write!(writer, "goodbye found")
            }).unwrap();

        let mut w = provider.text_writer("foo.txt");
        write!(w, "hello world").unwrap();
    });

    assert_eq!(
        results,
        vec![Result::Difference(Tripple {
            actual: "/actual/hi/foo.txt".into(),
            expected: "/expected/hi/foo.txt".into(),
            diffs: vec!["/diff/hi/foo.txt.diff".into()],
        })]
    );
}
