#![doc = "The closed code set every crate maps its refusals to. A code is the whole verdict vocabulary: \
its wire name, whether it refuses or is inconclusive, whether retrying can help, and what to tell the \
holder. `spec` holds those words."]

mod spec;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerdictKind {
    Refused,
    Inconclusive,
}

impl fmt::Display for VerdictKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            VerdictKind::Refused => "REFUSED",
            VerdictKind::Inconclusive => "INCONCLUSIVE",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    ManifestInvalid,
    WindowInvalid,
    ContextMismatch,
    RootMismatch,
    ProofInvalid,
    BadSignature,
    ChallengeUnknownOrExpired,
    AlreadyUsedInContext,
    RootSourcesDisagree,
    RootSourceUnavailable,
    WalletNotSyncedPastH1,
    CoinsTooNew,
    InsufficientHeldBalance,
    PirUnavailable,
    TooManyNotesNeeded,
    TreeStateUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeSpec {
    pub wire_name: &'static str,
    pub kind: VerdictKind,
    pub retryable: bool,
    pub message: &'static str,
    pub next_action: &'static str,
}

impl Code {
    pub const ALL: [Code; 16] = [
        Code::ManifestInvalid,
        Code::WindowInvalid,
        Code::ContextMismatch,
        Code::RootMismatch,
        Code::ProofInvalid,
        Code::BadSignature,
        Code::ChallengeUnknownOrExpired,
        Code::AlreadyUsedInContext,
        Code::RootSourcesDisagree,
        Code::RootSourceUnavailable,
        Code::WalletNotSyncedPastH1,
        Code::CoinsTooNew,
        Code::InsufficientHeldBalance,
        Code::PirUnavailable,
        Code::TooManyNotesNeeded,
        Code::TreeStateUnavailable,
    ];
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.spec().wire_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{code}: {detail}")]
pub struct Refusal {
    pub code: Code,
    pub detail: String,
}

impl Refusal {
    pub fn new(code: Code, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}
