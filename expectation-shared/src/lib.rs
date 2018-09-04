#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate walkdir;

pub mod filesystem;

use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Double {
    pub actual: PathBuf,
    pub expected: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Difference {
    pub actual: PathBuf,
    pub expected: PathBuf,
    pub diffs: Vec<PathBuf>,
    pub html: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ResultKind {
    Ok,
    ExpectedNotFound(Double),
    ActualNotFound(Double),
    Difference(Difference),
    IoError(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Result {
    pub test_name: String,
    pub file_name: PathBuf,
    pub kind: ResultKind,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Message {
    TestStarted { name: String },
    TestFinished { name: String, result: Vec<Result> },
    TestPanicked { name: String, details: String },
}

impl Result {
    pub fn is_ok(&self) -> bool {
        match &self.kind {
            ResultKind::Ok => true,
            _ => false,
        }
    }

    pub fn ok<N, P>(name: N, file: P) -> Self
    where
        N: Into<String>,
        P: Into<PathBuf>,
    {
        Result {
            test_name: name.into(),
            file_name: file.into(),
            kind: ResultKind::Ok,
        }
    }

    pub fn expected_not_found<N, P1, P2, P3>(name: N, file: P1, actual: P2, expected: P3) -> Self
    where
        N: Into<String>,
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
        P3: Into<PathBuf>,
    {
        Result {
            test_name: name.into(),
            file_name: file.into(),
            kind: ResultKind::ExpectedNotFound(Double {
                actual: actual.into(),
                expected: expected.into(),
            }),
        }
    }

    pub fn actual_not_found<N, P1, P2, P3>(name: N, file: P1, actual: P2, expected: P3) -> Self
    where
        N: Into<String>,
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
        P3: Into<PathBuf>,
    {
        Result {
            test_name: name.into(),
            file_name: file.into(),
            kind: ResultKind::ActualNotFound(Double {
                actual: actual.into(),
                expected: expected.into(),
            }),
        }
    }

    pub fn difference<N, P1, P2, P3>(
        name: N,
        file: P1,
        actual: P2,
        expected: P3,
        diffs: Vec<PathBuf>,
        html: Option<String>,
    ) -> Self
    where
        N: Into<String>,
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
        P3: Into<PathBuf>,
    {
        Result {
            test_name: name.into(),
            file_name: file.into(),
            kind: ResultKind::Difference(Difference {
                actual: actual.into(),
                expected: expected.into(),
                diffs,
                html,
            }),
        }
    }

    pub fn io_error<N, P>(name: N, file: P, io_error: std::io::Error) -> Self
    where
        N: Into<String>,
        P: Into<PathBuf>,
    {
        Result {
            test_name: name.into(),
            file_name: file.into(),
            kind: ResultKind::IoError(format!("{:?}", io_error)),
        }
    }
}
