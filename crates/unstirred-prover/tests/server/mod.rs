//! A one-answer HTTP listener on 127.0.0.1, used to see exactly what the prover puts on the wire
//! and to make a PIR server fail or serve a chosen root. It is not an `nf-server`: it never
//! answers a PIR query.
#![allow(dead_code)]

use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Recorded = Arc<Mutex<Vec<Vec<u8>>>>;

pub struct FakeServer {
    port: u16,
    requests: Recorded,
}

impl FakeServer {
    /// Answers every request with `status` and `body`.
    pub fn answering(status: u16, body: &str) -> Result<Self, Box<dyn Error>> {
        Self::start(status, body.to_owned())
    }

    /// Answers `GET /root` the way an `nf-server` does, for a chosen root and height. Everything
    /// else gets the same answer, which no PIR client can use.
    pub fn serving_root(circuit_root: &str, height: u32) -> Result<Self, Box<dyn Error>> {
        let body = format!(
            r#"{{"zcash_network":"test","nullifier_pool":"ironwood","dataset_version":2,
                 "circuit_root":"{circuit_root}","pir_root":"{circuit_root}","num_ranges":276626,
                 "pir_layout":{{"pir_depth":19,"tier0_layers":12,"tier1_layers":7,"poly_len":4096}},
                 "pir_depth":19,"tier1_rows":4096,"tier1_row_bytes":12288,"height":{height}}}"#
        );
        Self::start(200, body)
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Every request the server received, as raw bytes.
    pub fn requests(&self) -> Vec<Vec<u8>> {
        match self.requests.lock() {
            Ok(requests) => requests.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    fn start(status: u16, body: String) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let requests: Recorded = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&requests);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                answer(stream, status, &body, &recorder);
            }
        });
        Ok(Self { port, requests })
    }
}

fn answer(mut stream: TcpStream, status: u16, body: &str, recorder: &Recorded) {
    let mut request = Vec::new();
    let mut chunk = [0u8; 8192];
    while let Ok(read) = stream.read(&mut chunk) {
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if headers_complete(&request) && body_complete(&request) {
            break;
        }
    }
    if let Ok(mut recorded) = recorder.lock() {
        recorded.push(request);
    }
    let response = format!(
        "HTTP/1.1 {status} \r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn headers_complete(request: &[u8]) -> bool {
    request.windows(4).any(|window| window == b"\r\n\r\n")
}

/// Enough of the body to see what was sent: the declared length, when there is one.
fn body_complete(request: &[u8]) -> bool {
    let text = String::from_utf8_lossy(request);
    let Some((headers, body)) = text.split_once("\r\n\r\n") else {
        return false;
    };
    match declared_length(headers) {
        Some(length) => body.len() >= length,
        None => true,
    }
}

fn declared_length(headers: &str) -> Option<usize> {
    headers
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("content-length:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|value| value.trim().parse().ok())
}
