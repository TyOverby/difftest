use colored::*;
use expectation_shared::{Result as EResult, ResultKind};

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

    if !passed || verbose {
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
}
