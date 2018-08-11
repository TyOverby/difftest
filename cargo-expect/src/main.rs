#[macro_use]
extern crate structopt;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Specifier {
    /// Specifies which tests to run or promote
    #[structopt(name = "filter")]
    filter: Option<String>,

    /// Filetypes is a filter for which kinds of files are considered
    /// when running tests and promoting results.
    #[structopt(short = "f", long = "filetypes")]
    filetypes: Vec<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = r#"EXAMPLES:
    my-cli-tool run                     # runs all tests
    my-cli-tool run -f svg              # runs all tests but only diffs svg files
    my-cli-tool run my_test_name        # uses "my_test_name" as a filter for running tests
    my-cli-tool run my_test_name -f svg # uses "my_test_name" as a filter for running tests but only diffs svg files
    my-cli-tool promote    # promotes all tests with all files
    my-cli-tool promote -f svg  # promotes all tests but only promotes svg files produced by those tests
    my-cli-tool promote my_test_name   # promotes all files in tests that match "my_test_name"
    my-cli-tool promote my_test_name -f svg  # promotes only svg files for tests that match "my_test_name"
"#
)]
enum Command {
    /// Runs expectation tests in this crate
    #[structopt(name = "run")]
    Run(Specifier),

    /// Promotes the "actual" files to "expected" files
    #[structopt(name = "promote")]
    Promote(Specifier),

    /// Cleans up the expectation-tests directory by removing the "diff" and "actual" folders.
    #[structopt(name = "clean")]
    Clean,
}

fn main() {
    let command = Command::from_args();
    println!("{:?}", command);
}
