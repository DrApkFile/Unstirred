//! Reading a gate's answers and its log in a test: the one verdict a response carries, the JSON a
//! listing or a room log returned, and the scan for an address in a log line.

use std::error::Error;

use unstirred_gate::{Answer, GateVerdict};

pub fn expect_verdict(answer: &Answer, status: u16, verdict: &str) -> Result<GateVerdict, Box<dyn Error>> {
    if answer.status != status {
        return Err(format!(
            "expected HTTP {status} with {verdict}, the gate answered {} with {}",
            answer.status, answer.body
        )
        .into());
    }
    let answered = answer
        .verdict()
        .ok_or_else(|| format!("the gate answered no verdict object: {}", answer.body))?;
    if answered.verdict != verdict {
        return Err(format!("expected {verdict}, the gate answered {}", answer.body).into());
    }
    Ok(answered)
}

pub fn expect_code(answer: &Answer, status: u16, verdict: &str, code: &str) -> Result<(), Box<dyn Error>> {
    let answered = expect_verdict(answer, status, verdict)?;
    match answered.code.as_deref() {
        Some(answered_code) if answered_code == code => Ok(()),
        other => Err(format!("expected code {code}, the gate answered {other:?}").into()),
    }
}

/// An HTTP-level refusal: a status, a message, and no verdict at all. A gate that never read a
/// package has decided nothing, so there is nothing for it to name a code for (A37, A38).
pub fn expect_no_verdict(answer: &Answer, status: u16) -> Result<String, Box<dyn Error>> {
    if answer.status != status {
        return Err(format!(
            "expected HTTP {status}, the gate answered {} with {}",
            answer.status, answer.body
        )
        .into());
    }
    if answer.body.contains("\"verdict\"") || answer.body.contains("\"code\"") {
        return Err(format!(
            "an HTTP-level refusal must claim no verdict and no code: {}",
            answer.body
        )
        .into());
    }
    answer
        .error()
        .ok_or_else(|| format!("expected an error body, the gate answered {}", answer.body).into())
}

pub fn json_of(answer: &Answer) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&answer.body)?)
}

/// Whether a line contains something shaped like an IPv4 address. The gate prints one only at
/// startup, for the address it bound itself (A39).
pub fn contains_address(line: &str) -> bool {
    let bytes: Vec<&str> = line
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter(|part| !part.is_empty())
        .collect();
    bytes.iter().any(|part| {
        let groups: Vec<&str> = part.split('.').collect();
        groups.len() == 4
            && groups
                .iter()
                .all(|group| !group.is_empty() && group.parse::<u8>().is_ok())
    })
}

pub fn require_absent_from_log(lines: &[String], needle: &str, label: &str) -> Result<(), Box<dyn Error>> {
    if needle.is_empty() {
        return Err(format!("{label} was empty, so its absence proves nothing").into());
    }
    match lines.iter().find(|line| line.contains(needle)) {
        Some(line) => Err(format!("{label} appears in the gate's log: {line}").into()),
        None => Ok(()),
    }
}
