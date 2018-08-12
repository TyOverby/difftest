use serde_json;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use expectation_shared::Result as EResult;

fn get_stream() -> Option<TcpStream> {
    let env_var = match env::var("CARGO_EXPECT_IPC") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            return None;
        }
    };

    let stream = match TcpStream::connect(env_var) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            return None;
        }
    };

    Some(stream)
}

pub fn send(messages: &[EResult]) {
    if let Some(mut s) = get_stream() {
        for message in messages {
            serde_json::to_writer_pretty(&mut s, message).unwrap();
        }
    }
}
