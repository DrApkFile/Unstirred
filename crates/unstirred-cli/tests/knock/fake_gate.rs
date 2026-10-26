//! A gate that only answers manifests, and writes down every path it was asked for. It is not
//! the real service — `unstirred-gate`'s own tests are that — it is a recorder, so a test can say
//! which routes a knock touched and which it never went near.
//!
//! It also builds the wallet those knocks are run against: the synthetic wallet from
//! `unstirred_prover::testing`, written to a directory the real binary opens read-only.

use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use unstirred_prover::testing::{NoteA, SyntheticWallet, H1};

pub type TestResult = Result<(), Box<dyn Error>>;

/// The gate's own binary, built by cargo for this test.
const UNSTIRRED: &str = env!("CARGO_BIN_EXE_unstirred");

/// Every path the knock asked this gate for, in order.
#[derive(Clone, Default)]
pub struct Asked(Arc<Mutex<Vec<String>>>);

impl Asked {
    pub fn paths(&self) -> Vec<String> {
        match self.0.lock() {
            Ok(paths) => paths.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    fn record(&self, path: String) {
        match self.0.lock() {
            Ok(mut paths) => paths.push(path),
            Err(poisoned) => poisoned.into_inner().push(path),
        }
    }
}

/// A listener that answers `GET /rooms/<context>/manifest` for the rooms it was given and 404 for
/// everything else, recording every request. A challenge or an admission is therefore both
/// refused and visible: a test can prove a knock never asked for one.
pub struct FakeGate {
    pub url: String,
    pub asked: Asked,
}

/// The two service addresses this recorder advertises in `GET /rooms`, for the knock that leaves
/// a flag out (A50). Neither is reachable, and neither needs to be: what is under test is which
/// address the command chose, which it prints before it goes anywhere.
pub const ADVERTISED_PIR: &str = "http://advertised-nf-server.invalid:8081";
pub const ADVERTISED_LIGHTWALLETD: &str = "https://advertised-lightwalletd.invalid:443";

impl FakeGate {
    /// A recorder that advertises nothing, as a gate written before this existed does.
    pub fn serving(rooms: &[(&str, u32, u32)]) -> Result<Self, Box<dyn Error>> {
        Self::start(rooms, false)
    }

    /// A recorder whose `GET /rooms` carries both service addresses.
    pub fn advertising(rooms: &[(&str, u32, u32)]) -> Result<Self, Box<dyn Error>> {
        Self::start(rooms, true)
    }

    fn start(rooms: &[(&str, u32, u32)], advertise: bool) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))?;
        let url = format!("http://{}", listener.local_addr()?);
        let asked = Asked::default();
        let mut routes: Vec<(String, String)> = rooms
            .iter()
            .map(|(context, h0, h1)| (format!("/rooms/{context}/manifest"), manifest_json(context, *h0, *h1)))
            .collect();
        routes.push(("/rooms".to_owned(), rooms_json(advertise)));
        let answering = asked.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = answer(stream, &routes, &answering);
            }
        });
        Ok(Self { url, asked })
    }
}

fn answer(mut stream: TcpStream, routes: &[(String, String)], asked: &Asked) -> Result<(), Box<dyn Error>> {
    let path = read_request_path(&mut stream)?;
    asked.record(path.clone());
    let body = routes
        .iter()
        .find(|(route, _)| *route == path)
        .map(|(_, json)| json.clone());
    let (status, body) = match body {
        Some(json) => ("200 OK", json),
        None => (
            "404 Not Found",
            "{\"error\":\"this recorder answers the room list and manifests only\"}".to_owned(),
        ),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

/// The request line's path, read without a HTTP library: everything up to the blank line, then
/// the second word of the first line. A body is neither read nor needed — a knock that posts one
/// has already failed the test it is in.
fn read_request_path(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    let mut seen = Vec::new();
    let mut byte = [0u8; 1];
    while !seen.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte)? == 0 {
            break;
        }
        seen.push(byte[0]);
    }
    let text = String::from_utf8_lossy(&seen);
    let first = text.lines().next().unwrap_or_default();
    let path = first.split_whitespace().nth(1).unwrap_or_default();
    Ok(path.to_owned())
}

/// The room list, with or without the `services` a gate may advertise. The rooms themselves are
/// left out: a knock reads this route for the advertisement and for nothing else.
fn rooms_json(advertise: bool) -> String {
    if advertise {
        format!(
            r#"{{"rooms":[],"root_sources":[],"services":{{"pir_url":"{ADVERTISED_PIR}","lightwalletd_url":"{ADVERTISED_LIGHTWALLETD}"}}}}"#
        )
    } else {
        r#"{"rooms":[],"root_sources":[]}"#.to_owned()
    }
}

fn manifest_json(context: &str, h0: u32, h1: u32) -> String {
    format!(
        r#"{{"version":"unstirred/v1","network":"test","context":"{context}","window":{{"h0":{h0},"h1":{h1}}},"min_zatoshis":"12500000"}}"#
    )
}

/// A wallet the binary will open and refuse to read. **Synthetic:** its notes, its tree and its
/// heights are built in this process, and its keys are for a regtest network, so a command driven
/// by a gate manifest — which can only say `test` or `main` — reads it far enough to find that out
/// and stops. It is a real wallet database, opened read-only, and the step it fails is the local
/// one: exactly the step these tests are about.
pub fn unreadable_wallet(directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let wallet = directory.join("wallet");
    std::fs::create_dir_all(&wallet)?;
    SyntheticWallet::build(NoteA::StaysPut)?.write_to(&wallet)?;
    Ok(wallet)
}

/// The height the synthetic wallet is scanned through, which every window in these tests ends at.
pub const SCANNED_THROUGH: u32 = H1;

/// What the binary printed and what it exited with. stdout and stderr are kept apart because the
/// knock puts the holder's own lines on one and the gate's answer on the other in `--json` mode.
pub struct Ran {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Ran {
    pub fn said(&self, text: &str) -> bool {
        self.stdout.contains(text) || self.stderr.contains(text)
    }
}

pub fn run_unstirred(args: &[&str]) -> Result<Ran, Box<dyn Error>> {
    let output = Command::new(UNSTIRRED).args(args).output()?;
    Ok(Ran {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
