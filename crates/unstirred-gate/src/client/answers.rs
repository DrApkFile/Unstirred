#![doc = "What the gate's answers look like to a holder: a challenge with its expiry, and the PRD \
section 3.5 verdict. A body that is not the shape the route promises is an error, never a guess."]

use serde::Deserialize;

use super::GateClientError;

/// One answer, exactly as the gate gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: String,
}

impl Answer {
    /// The verdict in this answer, when the body is one. A `{"error": …}` body is not a verdict.
    pub fn verdict(&self) -> Option<GateVerdict> {
        serde_json::from_str(&self.body).ok()
    }

    /// The HTTP-level error message, when the gate answered with one instead of a verdict.
    pub fn error(&self) -> Option<String> {
        #[derive(Deserialize)]
        struct Problem {
            error: String,
        }
        serde_json::from_str::<Problem>(&self.body)
            .ok()
            .map(|problem| problem.error)
    }
}

/// The PRD section 3.5 verdict object. `code`, `message` and `retryable` are absent on ADMITTED.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GateVerdict {
    pub verdict: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub retryable: Option<bool>,
    #[serde(default)]
    pub note: Option<String>,
}

impl GateVerdict {
    pub fn admitted(&self) -> bool {
        self.verdict == "ADMITTED"
    }
}

/// A challenge the gate issued, with the expiry it stored for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedChallenge {
    pub raw: [u8; 32],
    pub expires_at: String,
}

#[derive(Deserialize)]
struct ChallengeBody {
    challenge: String,
    expires_at: String,
}

pub(super) fn read_challenge(body: &str, path: &str) -> Result<IssuedChallenge, GateClientError> {
    let answered: ChallengeBody = serde_json::from_str(body).map_err(|e| GateClientError::Unreadable {
        path: path.to_owned(),
        detail: format!("{e}"),
    })?;
    let decoded = hex::decode(answered.challenge.trim()).map_err(|e| GateClientError::Unreadable {
        path: path.to_owned(),
        detail: format!("the challenge is not hex: {e}"),
    })?;
    let raw = <[u8; 32]>::try_from(decoded.as_slice()).map_err(|_| GateClientError::Unreadable {
        path: path.to_owned(),
        detail: format!("the challenge is {} bytes, not 32", decoded.len()),
    })?;
    Ok(IssuedChallenge {
        raw,
        expires_at: answered.expires_at,
    })
}
