use super::Specifier;
use colored::*;
use crossbeam::channel::{unbounded, Receiver};
use expectation_shared::filesystem::*;
use expectation_shared::Message;
use promote::promote;
use serde_json;
use std::io::Result as IoResult;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread::spawn;

fn get_listener() -> IoResult<TcpListener> {
    for i in 0..100 {
        let port = 9000 + i;
        match TcpListener::bind(format!("localhost:{}", port)) {
            Ok(l) => return Ok(l),
            Err(_) => continue,
        }
    }
    TcpListener::bind("localhost:{9100}")
}

pub fn tcp_listen() -> IoResult<(String, Receiver<Message>)> {
    let listener = get_listener()?;
    let addr = listener.local_addr();
    let (sender, receiver) = unbounded();

    spawn(move || loop {
        let sender = sender.clone();
        match listener.accept() {
            Ok((conn, _)) => {
                spawn(move || match serde_json::from_reader(conn) {
                    Ok(out) => {
                        let _ = sender.send(out);
                    }
                    Err(e) => eprintln!("{}", e),
                });
            }
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            }
        }
    });
    Ok((format!("{}", addr?), receiver))
}

pub fn process_listen(mut command: Command) -> IoResult<Receiver<()>> {
    let (sender, receiver) = unbounded();
    let mut handle = command.spawn()?;
    spawn(move || {
        let _ = handle.wait();
        let _ = sender.send(());
    });

    Ok(receiver)
}

fn prepare_command(spec: Specifier, send_ser: String) -> Command {
    let mut command = Command::new("cargo");
    command.arg("test");
    command.arg("--lib");
    if spec.release {
        command.arg("--release");
    }
    command.arg("expectation_test");
    if let Some(filter) = spec.filter {
        command.env("CARGO_EXPECT_FILTER", filter);
    }
    if !spec.filetypes.is_empty() {
        command.env("CARGO_EXPECT_FILES", spec.filetypes.join(","));
    }
    command.env("CARGO_EXPECT_IPC", send_ser);
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command
}

fn run_build(release: bool) -> IoResult<ExitStatus> {
    let mut command = Command::new("cargo");
    command.arg("build");
    command.arg("--lib");
    command.arg("--tests");
    if release {
        command.arg("--release");
    }
    println!("Building Library");
    let result = command.spawn()?.wait()?;
    Ok(result)
}

pub fn fold_wait<F, R>(messages: Receiver<Message>, done: Receiver<()>, mut init: R, f: F) -> R
where
    F: Fn(R, Message) -> R,
{
    'a: loop {
        select![
            recv(messages, item) => {
                match item {
                    Some(message) => {
                        init = f(init, message);
                    },
                    None => { break 'a; }
                }
            },
            recv(done, _) => { break 'a; }
        ]
    }

    loop {
        if let Some(message) = messages.try_recv() {
            init = f(init, message);
        } else {
            break;
        }
    }

    init
}

pub fn perform_promote(spec: Specifier) -> IoResult<bool> {
    if !run_build(spec.release)?.success() {
        return Ok(false);
    }
    println!("Promoting Library");

    let verbose = spec.verbose;
    let (send_ser, messages) = tcp_listen()?;
    let command = prepare_command(spec, send_ser);
    let done_recvr = process_listen(command)?;

    let fs = RealFileSystem { root: "/".into() };

    let (success, files_promoted_count) = fold_wait(
        messages,
        done_recvr,
        (true, 0),
        |(mut success, mut files_promoted_count), message| {
            match message {
                Message::TestFinished { name, result } => {
                    let rs: Vec<_> = result
                        .into_iter()
                        .map(|r| {
                            let p = promote(&r.kind, fs.duplicate());
                            (r, p)
                        }).collect();
                    let (s, c_count) = ::output::print_promotion(&name, rs, verbose);
                    success &= s;
                    files_promoted_count += c_count;
                }
                _ => unimplemented!(),
            }
            (success, files_promoted_count)
        },
    );

    println!("{} Files Promoted", files_promoted_count);

    Ok(success)
}

pub fn perform_run(spec: Specifier) -> IoResult<bool> {
    if !run_build(spec.release)?.success() {
        return Ok(false);
    }
    println!("Running Library");

    let verbose = spec.verbose;
    let (send_ser, messages) = tcp_listen()?;
    let command = prepare_command(spec, send_ser);
    let done_recvr = process_listen(command)?;

    let mut total_results = fold_wait(
        messages,
        done_recvr,
        vec![],
        |mut total_results, message| {
            match message {
                Message::TestFinished { name, result } => {
                    ::output::print_results(&name, &result, verbose);
                    total_results.push((name, result, true));
                }
                _ => unimplemented!(),
            }
            total_results
        },
    );

    let mut total_suites = 0;
    let mut failed_suites = 0;
    let mut total_files = 0;
    let mut failed_files = 0;

    for (_, results, passed) in &mut total_results {
        total_suites += 1;
        let mut success = true;
        for file in results {
            total_files += 1;
            if !file.is_ok() {
                failed_files += 1;
                success = false;
                *passed = false;
            }
        }
        if !success {
            failed_suites += 1;
        }
    }

    let colorizer = |s: &str| {
        if failed_suites == 0 {
            s.green()
        } else {
            s.red()
        }
    };

    println!("{}︎ Expectation Results", colorizer("◼"));
    println!(
        "  {} Tests: {} / {}",
        colorizer("►"),
        total_suites - failed_suites,
        total_suites
    );
    println!(
        "  {} Files: {} / {}",
        colorizer("►"),
        total_files - failed_files,
        total_files
    );

    let fs = RealFileSystem { root: "./".into() };
    fs.write(Path::new("./out.html"), &mut |w| {
        super::html::format_html(&total_results, w)
    })?;

    Ok(failed_suites == 0)
}
