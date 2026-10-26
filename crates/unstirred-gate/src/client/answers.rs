#![doc = "What the gate's answers look like to a holder: a challenge with its expiry, and the PRD \
section 3.5 verdict. A body that is not the shape the route promises is an error, never a guess."]

use serde::Deserialize;

use super::GateClientError;

/// One answer, exactly as the gate gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// The HTTP status, unmodified. A refusal is a 200 with a verdict body, so a non-200 is a
    /// transport or routing problem rather than a verdict.
    pub status: u16,
    /// The body, as text.
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
    /// `ADMITTED`, `REFUSED` or `INCONCLUSIVE`.
    pub verdict: String,
    /// The code from the closed set, on a refusal or an inconclusive answer.
    #[serde(default)]
    pub code: Option<String>,
    /// What the holder is told, in the code's own words.
    #[serde(default)]
    pub message: Option<String>,
    /// Whether trying again could change the answer.
    #[serde(default)]
    pub retryable: Option<bool>,
    /// The gate's standing note, such as the single-root-source label.
    #[serde(default)]
    pub note: Option<String>,
}

impl GateVerdict {
    /// Whether this verdict admits. Anything else, including an inconclusive answer, does not.
    pub fn admitted(&self) -> bool {
        self.verdict == "ADMITTED"
    }
}

/// What a gate advertises in `GET /rooms` for the holder's own side: the public service addresses
/// its operator runs. A gate that advertises neither answers without the field, which reads here
/// as two `None`s and is not an error (A49).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct AdvertisedServices {
    /// The nullifier-PIR server this gate suggests for the private lookup at h1.
    #[serde(default)]
    pub pir_url: Option<String>,
    /// The lightwalletd this gate suggests for the tree state at h0.
    #[serde(default)]
    pub lightwalletd_url: Option<String>,
}

#[derive(Deserialize)]
struct Listing {
    #[serde(default)]
    services: Option<AdvertisedServices>,
}

/// The advertisement in a `GET /rooms` body. A body that carries no `services` is a gate that
/// advertises nothing, which is not a failure: the holder is simply told to pass the flags.
pub(super) fn read_services(body: &str, path: &str) -> Result<AdvertisedServices, GateClientError> {
    let listing: Listing = serde_json::from_str(body).map_err(|e| GateClientError::Unreadable {
        path: path.to_owned(),
        detail: format!("{e}"),
    })?;
    Ok(listing.services.unwrap_or_default())
}

/// A challenge the gate issued, with the expiry it stored for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedChallenge {
    /// The challenge bytes to bind a package to.
    pub raw: [u8; 32],
    /// When the gate will stop accepting it, as the gate wrote it.
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
