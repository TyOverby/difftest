use super::Specifier;
use colored::*;
use crossbeam::channel::{unbounded, Receiver};
use expectation_shared::Result as EResult;
use serde_json;
use std::io::Result as IoResult;
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread::spawn;

fn get_listener() -> IoResult<TcpListener> {
    for i in 0..100 {
        let port = 9000 + i;
        match TcpListener::bind(format!("localhost:{}", port)) {
            Ok(l) => return Ok(l),
            Err(_) => continue,
        }
    }
    TcpListener::bind("localhost:{9101}")
}

pub fn tcp_listen() -> IoResult<(String, Receiver<(String, Vec<EResult>)>)> {
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
    Ok((format!("{}", addr.unwrap()), receiver))
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

pub fn perform_run(spec: Specifier) -> bool {
    let verbose = spec.verbose;
    let (send_ser, messages) = tcp_listen().unwrap();
    let command = prepare_command(spec, send_ser);
    let done_recvr = process_listen(command);

    let mut total_results = vec![];

    'a: loop {
        select![
            recv(messages, item) => {
                match item {
                    Some((name, results)) => {
                        ::output::print_results(&name, &results, verbose);
                        total_results.push((name, results));
                    },
                    None => { break 'a; }
                }
            },
            recv(done_recvr, _) => { break 'a; }
        ]
    }

    loop {
        if let Some((name, results)) = messages.try_recv() {
            ::output::print_results(&name, &results, verbose);
            total_results.push((name, results));
        } else {
            break;
        }
    }

    let mut total_suites = 0;
    let mut failed_suites = 0;
    let mut total_files = 0;
    let mut failed_files = 0;

    for (_, results) in total_results {
        total_suites += 1;
        let mut success = true;
        for file in results {
            total_files += 1;
            if !file.is_ok() {
                failed_files += 1;
                success = false;
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

    failed_suites == 0
}
