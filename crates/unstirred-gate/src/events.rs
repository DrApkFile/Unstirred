#![doc = "Everything a gate says about itself. One line per request: the method, the path, the \
status and the verdict code when there was a verdict. No peer address, no request body, no raw \
challenge, no session key, and no marker: the only address a gate ever prints is its own listen \
address, at startup (LEAKS L9, A39). There is no middleware here that could add one."]

use std::sync::{Arc, Mutex};

/// Where a gate's lines go. Implementations must not add anything to a line.
pub trait GateLog: std::fmt::Debug + Send + Sync {
    fn line(&self, line: &str);
}

/// The log a served gate uses.
#[derive(Debug, Clone, Copy)]
pub struct StdoutLog;

impl GateLog for StdoutLog {
    fn line(&self, line: &str) {
        println!("{line}");
    }
}

/// A log a test can read back, so A39 asserts the rendered lines rather than trusting them.
#[derive(Debug, Clone, Default)]
pub struct CollectedLog {
    lines: Arc<Mutex<Vec<String>>>,
}

impl CollectedLog {
    pub fn lines(&self) -> Vec<String> {
        match self.lines.lock() {
            Ok(lines) => lines.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl GateLog for CollectedLog {
    fn line(&self, line: &str) {
        match self.lines.lock() {
            Ok(mut lines) => lines.push(line.to_owned()),
            Err(poisoned) => poisoned.into_inner().push(line.to_owned()),
        }
    }
}

/// One answered request, as the gate records it. `code` is the verdict code when the answer was
/// a verdict, and nothing otherwise: an HTTP-level refusal claims no code from the closed set.
pub fn request_line(method: &str, path: &str, status: u16, code: Option<&str>) -> String {
    match code {
        Some(code) => format!("{method} {path} -> {status} {code}"),
        None => format!("{method} {path} -> {status}"),
    }
}
