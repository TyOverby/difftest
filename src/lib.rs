pub mod diff;
mod filesystem;

#[cfg(test)]
use filesystem::FakeFileSystem;
use filesystem::FileSystem;
use filesystem::RealFileSystem;
use std::path::PathBuf;

type ComputeFunction = Fn(Vec<u8>) -> Vec<(PathBuf, Vec<u8>)> + 'static;
type EqFunction = Fn(&[u8], &[u8]) -> bool + 'static;
type DiffFunction = Fn(Vec<u8>, Vec<u8>) -> (String, Vec<u8>) + 'static;

struct Strategy {
    compute: Box<ComputeFunction>,
    eq: Box<EqFunction>,
    diff: Box<DiffFunction>,
}

pub struct TestSuite {
    strategies: Vec<(Strategy, Vec<PathBuf>)>,
}

pub struct StrategyBuilder<'a> {
    function: Strategy,
    input_files: Vec<PathBuf>,
    suite: &'a mut TestSuite,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TestResult {
    Good {
        input: PathBuf,
        output: PathBuf,
    },
    InputNotFound(PathBuf),
    OutputNotFound(PathBuf),
    IoError {
        input: PathBuf,
        output: PathBuf,
        error: String,
    },
    Bad {
        actual: PathBuf,
        expected: PathBuf,
        diff: PathBuf,
    },
}

impl TestSuite {
    pub fn new() -> TestSuite {
        TestSuite { strategies: vec![] }
    }

    pub fn with_strategy<C: 'static, E: 'static, D: 'static>(
        &mut self,
        compute: C,
        eq: E,
        diff: D,
    ) -> StrategyBuilder
    where
        C: Fn(Vec<u8>) -> Vec<(PathBuf, Vec<u8>)>,
        E: Fn(&[u8], &[u8]) -> bool,
        D: Fn(Vec<u8>, Vec<u8>) -> (String, Vec<u8>),
    {
        StrategyBuilder {
            function: Strategy {
                compute: Box::new(compute),
                eq: Box::new(eq),
                diff: Box::new(diff),
            },
            input_files: vec![],
            suite: self,
        }
    }

    pub fn run(self, root: &str) -> Vec<TestResult> {
        self.run_test(&mut RealFileSystem { root: root.into() })
    }

    fn run_test<F: FileSystem>(self, fs: &mut F) -> Vec<TestResult> {
        let mut test_results = vec![];
        let input_subsystem = fs.subsystem("input");
        let actual_subsystem = fs.subsystem("actual");
        let expected_subsystem = fs.subsystem("expected");
        let diff_subsystem = fs.subsystem("diff");

        for (Strategy { compute, eq, diff }, inputs) in self.strategies {
            for input_file in inputs {
                let mut actual_subsystem = actual_subsystem.subsystem(&input_file);
                let expected_subsystem = expected_subsystem.subsystem(&input_file);
                let mut diff_subsystem = diff_subsystem.subsystem(&input_file);

                let contents = match input_subsystem.read(&input_file) {
                    Ok(c) => c,
                    Err(_) => {
                        test_results.push(TestResult::InputNotFound(input_file.clone()));
                        continue;
                    }
                };

                for (res_path, res_contents) in compute(contents) {
                    if let Err(e) = actual_subsystem.write(&res_path, &res_contents) {
                        test_results.push(TestResult::IoError {
                            input: input_file.clone(),
                            output: actual_subsystem.full_path_for(&res_path),
                            error: e.to_string(),
                        });
                    }

                    let act_contents = match expected_subsystem.read(&res_path) {
                        Ok(c) => c,
                        Err(_) => {
                            test_results.push(TestResult::OutputNotFound(
                                expected_subsystem.full_path_for(res_path),
                            ));
                            continue;
                        }
                    };

                    if eq(&act_contents, &res_contents) {
                        test_results.push(TestResult::Good {
                            input: input_file.clone(),
                            output: res_path.clone(),
                        });
                    } else {
                        let (diff_name, diff_contents) = diff(act_contents, res_contents);
                        diff_subsystem.write(&diff_name, &diff_contents).unwrap();
                        test_results.push(TestResult::Bad {
                            actual: actual_subsystem.full_path_for(&res_path),
                            expected: expected_subsystem.full_path_for(&res_path),
                            diff: diff_subsystem.full_path_for(&diff_name),
                        });
                    }
                }
            }
        }

        return test_results;
    }
}

impl<'a> StrategyBuilder<'a> {
    pub fn build(self) {
        self.suite
            .strategies
            .push((self.function, self.input_files));
    }

    pub fn input<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.input_files.push(path.into());
        self
    }
}

#[test]
fn simple_copy() {
    let mut fs = FakeFileSystem::new();
    fs.write("input/aa.txt", &[0, 1, 2, 3]).unwrap();

    let mut suite = TestSuite::new();
    let compute = |mut a: Vec<u8>| {
        vec![
            ("aa.out".into(), a.clone()),
            ("aa.rev".into(), {
                a.reverse();
                a
            }),
        ]
    };
    suite
        .with_strategy(compute, |a, b| a == b, |_, _| unimplemented!())
        .input("aa.txt")
        .build();
    let results = suite.run_test(&mut fs);

    assert_eq!(
        results,
        vec![
            TestResult::OutputNotFound("/expected/aa.txt/aa.out".into()),
            TestResult::OutputNotFound("/expected/aa.txt/aa.rev".into()),
        ]
    );

    assert!(fs.exists("actual/aa.txt/aa.out"));
    assert_eq!(fs.read("actual/aa.txt/aa.out").unwrap(), vec![0, 1, 2, 3]);

    assert!(fs.exists("actual/aa.txt/aa.rev"));
    assert_eq!(fs.read("actual/aa.txt/aa.rev").unwrap(), vec![3, 2, 1, 0]);
}

#[test]
fn diff_function_works() {
    let mut fs = FakeFileSystem::new();
    fs.write("input/aa.txt", &[0, 1, 2, 3]).unwrap();
    fs.write("expected/aa.txt/aa.out", &[0, 2, 2, 3]).unwrap();

    let mut suite = TestSuite::new();
    let compute = |a: Vec<u8>| vec![("aa.out".into(), a)];
    let diff = |mut a: Vec<u8>, b: Vec<u8>| {
        ("diff.bin".to_owned(), {
            a.extend(vec![0, 0]);
            a.extend(b);
            a
        })
    };
    suite
        .with_strategy(compute, |a, b| a == b, diff)
        .input("aa.txt")
        .build();

    let results = suite.run_test(&mut fs);
    assert_eq!(
        results,
        vec![TestResult::Bad {
            actual: "/actual/aa.txt/aa.out".into(),
            expected: "/expected/aa.txt/aa.out".into(),
            diff: "/diff/aa.txt/diff.bin".into(),
        }]
    );

    assert_eq!(
        fs.read("diff/aa.txt/diff.bin").unwrap(),
        vec![0, 2, 2, 3, 0, 0, 0, 1, 2, 3]
    );
}
