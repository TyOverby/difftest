use colored::*;
use expectation_shared::{Result as EResult, ResultKind};
use std::io::Result as IoResult;
use std::error::Error;

pub fn print_promotion(name: &str, results: Vec<(EResult, IoResult<String>)>, verbose: bool) -> bool {
    let passed = results
        .iter()
        .map(|&(_, ref b)| b)
        .all(|r|  r.is_ok());

    if passed {
        println!("︎{} {}", "✔".green(), name);
    } else {
        println!("{} {}", "✘".red(), name);
    }

    if passed && !verbose {
        return passed;
    }

    for (EResult{file_name, ..}, io_result) in results {
        match io_result {
            Ok(detail) => {
                println!(
                    "  {} {} ❯ Ok",
                    "✔".green(),
                    file_name.to_string_lossy()
                );
                if verbose {
                   println!("    ► {}", detail);
                }
            }
            Err(ioe) => {
                println!(
                    "  {} {} ❯ Error occurred during ",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!("    ► {}", ioe.description());
            }
        }
    }

    passed
}

pub fn print_results(name: &str, results: &[EResult], verbose: bool) {
    let passed = results.iter().all(|r| match r.kind {
        ResultKind::Ok => true,
        _ => false,
    });
    if passed {
        println!("︎{} {}", "✔".green(), name);
    } else {
        println!("{} {}", "✘".red(), name);
    }

    if passed && !verbose {
        return;
    }

    for result in results {
        match result {
            EResult {
                file_name,
                kind: ResultKind::Ok,
                ..
            } => {
                println!(
                    "  {}︎ {} ❯ Ok",
                    "✔".green(),
                    file_name.to_string_lossy()
                );
            }
            EResult {
                file_name,
                kind: ResultKind::ExpectedNotFound(double),
                ..
            } => {
                println!(
                    "  {} {} ❯ Expected Not Found",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!("    ► Actual: {}", double.actual.to_string_lossy());
                println!(
                    "    {} Expected: {}",
                    "☛".yellow(),
                    double.expected.to_string_lossy()
                );
            }
            EResult {
                file_name,
                kind: ResultKind::ActualNotFound(double),
                ..
            } => {
                println!(
                    "  {} {} ❯ Actual Not Found",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!(
                    "    {} Actual: {}",
                    "☛".yellow(),
                    double.actual.to_string_lossy()
                );
                println!("    ► Expected: {}", double.expected.to_string_lossy());
            }
            EResult {
                file_name,
                kind: ResultKind::Difference(tripple),
                ..
            } => {
                println!(
                    "  {} {} ❯ Difference",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!("    ► Actual: {}", tripple.actual.to_string_lossy());
                println!("    ► Expected: {}", tripple.expected.to_string_lossy());
                match tripple.diffs.len() {
                    0 => {}
                    1 => {
                        println!("    ► Diff: {}", tripple.diffs[0].to_string_lossy());
                    }
                    _ => {
                        println!("    ► Diffs:");
                        for diff in &tripple.diffs {
                            println!("      • {}", diff.to_string_lossy());
                        }
                    }
                }
            }
            EResult {
                file_name,
                kind: ResultKind::IoError(error),
                ..
            } => {
                println!(
                    "  {} Io Error for file {}: {}",
                    "✘".red(),
                    file_name.to_string_lossy(),
                    error
                );
            }
        }
    }
}
