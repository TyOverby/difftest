use super::Specifier;
use crossbeam::channel::{unbounded, Receiver};
use expectation_shared::Result as EResult;
use serde_json;
use serde_json::de::IoRead;
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

pub fn tcp_listen() -> IoResult<(String, Receiver<Vec<EResult>>)> {
    let listener = get_listener()?;
    let addr = listener.local_addr();

    let (sender, receiver) = unbounded();

    spawn(move || loop {
        let sender = sender.clone();
        match listener.accept() {
            Ok((conn, _)) => {
                spawn(move || {
                    let stream = serde_json::StreamDeserializer::new(IoRead::new(conn));
                    let out: Result<_, _> = stream.collect();
                    match out {
                        Ok(out) => {
                            let _ = sender.send(out);
                        }
                        Err(e) => eprintln!("{}", e),
                    }
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
    if let Some(filter) = spec.filter {
        command.arg(filter);
    }
    command.env("CARGO_EXPECT_IPC", send_ser);
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command
}

pub fn perform_run(spec: Specifier) {
    let (send_ser, messages) = tcp_listen().unwrap();
    let command = prepare_command(spec, send_ser);
    let done_recvr = process_listen(command);

    'a: loop {
        select![
            recv(messages, item) => {
                match item {
                    Some(item) => println!("{:?}", item),
                    None => { break 'a; }
                }
            },
            recv(done_recvr, _) => { break 'a; }
        ]
    }

    loop {
        if let Some(item) = messages.try_recv() {
            println!("{:#?}", item);
        } else {
            break;
        }
    }
}
