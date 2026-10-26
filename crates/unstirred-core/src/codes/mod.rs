#![doc = "The closed code set every crate maps its refusals to. A code is the whole verdict vocabulary: \
its wire name, whether it refuses or is inconclusive, whether retrying can help, and what to tell the \
holder. `spec` holds those words."]

mod spec;

use std::fmt;

/// Which of the two non-admitting outcomes a code belongs to. The distinction is not cosmetic:
/// a refusal is a decision about the holder, and an inconclusive answer is the gate saying it
/// could not decide at all. **Unknown is never success, and never failure.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerdictKind {
    /// A rule said no. The holder was judged.
    Refused,
    /// No judgement was reached — a source disagreed, or something was unreachable.
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

/// Every way this protocol can answer something other than ADMITTED. The set is **closed**: a
/// code cannot be added without also writing its message, its kind, its retryability and the
/// holder's next action, because [`CodeSpec`] is one `match` over this enum.
///
/// The first eight are a gate's; the last six are the holder's own side refusing before anything
/// is sent anywhere.
///
/// ```
/// use unstirred_core::{Code, VerdictKind};
///
/// // Every code carries its own words — there are no codes with nothing to say.
/// for code in Code::ALL {
///     assert!(!code.spec().message.is_empty());
///     assert!(!code.spec().next_action.is_empty());
/// }
///
/// // Coins used once at a room can never be used there again, so retrying cannot help.
/// assert!(!Code::AlreadyUsedInContext.spec().retryable);
/// // A gate that could not reach its sources might manage later.
/// assert_eq!(Code::RootSourceUnavailable.spec().kind, VerdictKind::Inconclusive);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    /// The manifest is malformed, or uses a version or threshold this build does not support.
    ManifestInvalid,
    /// `h0` is not below `h1`, `h1` is not a multiple of ten, or `h1` is not yet final.
    WindowInvalid,
    /// The proof was made for a different gate: its `vote_round_id` is not this context's.
    ContextMismatch,
    /// The proof is against roots this gate did not pin — the coins were not held across *this*
    /// window. This is what a newcomer's coins produce.
    RootMismatch,
    /// The Halo2 proof did not verify against the instance the package carries.
    ProofInvalid,
    /// The binding signature does not check out under `rk` rebuilt from the instance, so the
    /// prover does not control the coins. A viewing-key holder stops here.
    BadSignature,
    /// The challenge is unknown to this gate, already consumed, or past its expiry.
    ChallengeUnknownOrExpired,
    /// These coins have already been admitted at this context. The marker collision is the
    /// single-use rule, enforced by a `UNIQUE` constraint inside the admission transaction.
    AlreadyUsedInContext,
    /// Two root sources reported different h0 roots, so the gate has no root to check against.
    RootSourcesDisagree,
    /// A root source could not be reached, or answered for the wrong network or height.
    RootSourceUnavailable,
    /// The holder's wallet has not scanned past `h1`, so it cannot know the coins stayed unspent.
    WalletNotSyncedPastH1,
    /// None of the wallet's eligible coins existed at `h0`. No proof was attempted.
    CoinsTooNew,
    /// The coins held unmoved across the whole window total less than the threshold.
    InsufficientHeldBalance,
    /// The private nullifier lookup service is unavailable. **No plaintext query was made** and
    /// nothing was revealed — this is inconclusive, not a fallback.
    PirUnavailable,
    /// Reaching the threshold would need more than the five notes one proof can cover.
    TooManyNotesNeeded,
    /// The public tree state at `h0` could not be fetched, so no witness could be built.
    TreeStateUnavailable,
}

/// Everything one code means, in one place. A caller never writes its own wording for a verdict:
/// it takes these, so the same refusal reads the same way from the command line, from the HTTP
/// route and from a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeSpec {
    /// The name this code travels under in JSON and on a terminal, such as `ROOT_MISMATCH`.
    pub wire_name: &'static str,
    /// Whether this code refuses the holder or declines to decide.
    pub kind: VerdictKind,
    /// Whether another attempt could possibly produce a different answer.
    pub retryable: bool,
    /// What to tell the holder, in their terms rather than the protocol's.
    pub message: &'static str,
    /// What the holder can actually do about it.
    pub next_action: &'static str,
}

impl Code {
    /// Every code, in the order the specification lists them. `unstirred codes` prints this, and
    /// a test walks it to prove no code is missing its words.
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

/// One refusal: a code from the closed set, plus the detail of this particular occurrence. The
/// code carries the words shown to a holder; `detail` is for the operator and the log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{code}: {detail}")]
pub struct Refusal {
    /// Which code this is, and so what it means and whether it is worth retrying.
    pub code: Code,
    /// What happened this time — the height, the source, the field. Never a holder's secret.
    pub detail: String,
}

impl Refusal {
    /// A refusal of `code`, with the detail of this occurrence.
    ///
    /// ```
    /// use unstirred_core::{Code, Refusal};
    ///
    /// let refusal = Refusal::new(Code::WindowInvalid, "h1 4506511 is not a multiple of 10");
    /// assert_eq!(refusal.code, Code::WindowInvalid);
    /// // Displaying a refusal names the code and then the detail.
    /// assert!(refusal.to_string().starts_with("WINDOW_INVALID: "));
    /// ```
    pub fn new(code: Code, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}
