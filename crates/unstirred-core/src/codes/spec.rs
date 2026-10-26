#![doc = "What each code means: the wire name, the verdict kind, whether a retry can help, the message \
the holder reads and the action it points at. One match so a new code cannot be added without words."]

use super::{Code, CodeSpec, VerdictKind};

impl Code {
    /// This code's words: its wire name, its verdict kind, whether retrying can help, what the
    /// holder is told and what they can do. One `match`, so a code cannot exist without them.
    ///
    /// ```
    /// use unstirred_core::Code;
    /// assert_eq!(Code::RootMismatch.spec().wire_name, "ROOT_MISMATCH");
    /// ```
    pub const fn spec(self) -> CodeSpec {
        use VerdictKind::{Inconclusive, Refused};
        match self {
            Code::ManifestInvalid => CodeSpec {
                wire_name: "MANIFEST_INVALID",
                kind: Refused,
                retryable: false,
                message: "The gate manifest is malformed or uses unsupported values. Nothing was admitted.",
                next_action: "Ask the gate owner to publish a valid unstirred/v1 manifest.",
            },
            Code::WindowInvalid => CodeSpec {
                wire_name: "WINDOW_INVALID",
                kind: Refused,
                retryable: false,
                message:
                    "The holding window is not valid: h0 must be below h1, h1 a multiple of 10, and h1 final on chain.",
                next_action: "Wait until h1 is final, or ask the gate owner to fix the window.",
            },
            Code::ContextMismatch => CodeSpec {
                wire_name: "CONTEXT_MISMATCH",
                kind: Refused,
                retryable: false,
                message: "The proof was made for a different gate. Nothing was admitted.",
                next_action: "Prove again using this gate's manifest.",
            },
            Code::RootMismatch => CodeSpec {
                wire_name: "ROOT_MISMATCH",
                kind: Refused,
                retryable: false,
                message:
                    "The proof does not match this gate's window: the coins were not held unmoved from h0 through h1.",
                next_action: "Only coins held since h0 qualify. Nothing was admitted.",
            },
            Code::ProofInvalid => CodeSpec {
                wire_name: "PROOF_INVALID",
                kind: Refused,
                retryable: false,
                message: "The zero-knowledge proof did not verify. Nothing was admitted.",
                next_action: "Generate a fresh proof with an unmodified prover.",
            },
            Code::BadSignature => CodeSpec {
                wire_name: "BAD_SIGNATURE",
                kind: Refused,
                retryable: false,
                message: "The proof is not signed by the key that controls the coins. Nothing was admitted.",
                next_action: "Prove from the wallet that holds the spending key.",
            },
            Code::ChallengeUnknownOrExpired => CodeSpec {
                wire_name: "CHALLENGE_UNKNOWN_OR_EXPIRED",
                kind: Refused,
                retryable: true,
                message: "The challenge is unknown, already used or expired. Nothing was admitted.",
                next_action: "Request a new challenge and prove again.",
            },
            Code::AlreadyUsedInContext => CodeSpec {
                wire_name: "ALREADY_USED_IN_CONTEXT",
                kind: Refused,
                retryable: false,
                message: "These coins have already been used at this gate. Nothing was admitted.",
                next_action: "Each bundle of coins gets one admission per gate.",
            },
            Code::RootSourcesDisagree => CodeSpec {
                wire_name: "ROOT_SOURCES_DISAGREE",
                kind: Inconclusive,
                retryable: true,
                message: "The gate's root sources disagree, so it cannot decide. Nothing was admitted.",
                next_action: "Try again later; the gate owner should check its root sources.",
            },
            Code::RootSourceUnavailable => CodeSpec {
                wire_name: "ROOT_SOURCE_UNAVAILABLE",
                kind: Inconclusive,
                retryable: true,
                message: "The gate could not reach its root sources, so it cannot decide. Nothing was admitted.",
                next_action: "Try again later.",
            },
            Code::WalletNotSyncedPastH1 => CodeSpec {
                wire_name: "WALLET_NOT_SYNCED_PAST_H1",
                kind: Refused,
                retryable: true,
                message:
                    "The wallet has not synced past h1, so the prover cannot know whether the coins were still unspent.",
                next_action: "Sync the wallet past h1 and prove again.",
            },
            Code::CoinsTooNew => CodeSpec {
                wire_name: "COINS_TOO_NEW",
                kind: Refused,
                retryable: false,
                message: "None of the wallet's eligible coins existed at h0. No proof was made.",
                next_action: "Only coins held since h0 qualify for this gate.",
            },
            Code::InsufficientHeldBalance => CodeSpec {
                wire_name: "INSUFFICIENT_HELD_BALANCE",
                kind: Refused,
                retryable: false,
                message: "Coins held unmoved through the whole window total less than 0.125 ZEC. No proof was made.",
                next_action: "This gate needs at least 0.125 ZEC held since h0.",
            },
            Code::PirUnavailable => CodeSpec {
                wire_name: "PIR_UNAVAILABLE",
                kind: Inconclusive,
                retryable: true,
                message:
                    "The private nullifier lookup service is unavailable. No proof was made and nothing was revealed.",
                next_action: "Try again later. The prover never falls back to a plaintext query.",
            },
            Code::TreeStateUnavailable => CodeSpec {
                wire_name: "TREE_STATE_UNAVAILABLE",
                kind: Inconclusive,
                retryable: true,
                message: "Could not fetch the public tree state at h0. No proof was made and nothing was revealed.",
                next_action: "Try again later, or point the prover at another lightwalletd server.",
            },
            Code::TooManyNotesNeeded => CodeSpec {
                wire_name: "TOO_MANY_NOTES_NEEDED",
                kind: Refused,
                retryable: false,
                message:
                    "Reaching 0.125 ZEC would need more than 5 coins; one proof covers at most 5. No proof was made.",
                next_action: "This version cannot combine more than 5 coins.",
            },
        }
    }
}
