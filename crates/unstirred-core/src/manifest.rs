#![doc = "The gate manifest: the whole of what a gate publishes, and the whole of what a holder \
proves against. Five fields, parsed strictly — an unknown field, a version that is not \
`unstirred/v1`, a threshold that is not one ballot, or a context outside the charset is \
MANIFEST_INVALID. Nothing a gate could add here would be read."]

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::codes::{Code, Refusal};

/// The only protocol version this build speaks. A manifest or package claiming another is
/// refused rather than guessed at.
pub const PROTOCOL_VERSION: &str = "unstirred/v1";

/// The threshold, in zatoshis: 0.125 ZEC. It is one ballot in the delegation circuit, so it is
/// fixed by the circuit and is **not** a gate owner's to choose in v1.
pub const REQUIRED_MIN_ZATOSHIS: &str = "12500000";
const MAX_CONTEXT_LEN: usize = 64;

/// Which Zcash network a gate's rule is about. It is part of a gate's identity: `round_id` is
/// derived from the network and the context, so the same room name on two networks is two rooms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    /// Zcash testnet, which is the only network any recorded run in this repository used.
    Test,
    /// Zcash mainnet. Nothing here has been exercised against it.
    Main,
}

impl Network {
    /// The single byte that stands for this network inside a domain separator.
    ///
    /// ```
    /// use unstirred_core::Network;
    /// assert_eq!(Network::Test.domain_byte(), b't');
    /// assert_ne!(Network::Test.domain_byte(), Network::Main.domain_byte());
    /// ```
    pub const fn domain_byte(self) -> u8 {
        match self {
            Network::Test => b't',
            Network::Main => b'm',
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Network::Test => "test",
            Network::Main => "main",
        })
    }
}

/// The two block heights a gate's statement spans: held from `h0` through `h1`, unmoved. Pinning
/// the two roots at *different* heights is what makes this a duration proof rather than a
/// snapshot, and the rules an acceptable pair must meet are in [`crate::window`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    /// The height the note-commitment root is pinned at: the coins must already have existed here.
    pub h0: u32,
    /// The height the nullifier-set root is pinned at: the coins must still be unspent here.
    pub h1: u32,
}

/// One room's rule, as published. This is the only thing a holder needs from a gate in order to
/// prove, and the only thing of the gate's that the verifier reads when it checks a package —
/// never a value out of the package itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Always [`PROTOCOL_VERSION`]; any other value is refused at parse time.
    pub version: String,
    /// The network this rule is about.
    pub network: Network,
    /// The room's name, which scopes its markers. 1–64 characters of `a-z`, `0-9` and `-`,
    /// starting with a letter or a digit.
    pub context: String,
    /// The two heights the holding is proved across.
    pub window: Window,
    /// Always [`REQUIRED_MIN_ZATOSHIS`] in v1, as a decimal string.
    pub min_zatoshis: String,
}

/// Reads a manifest, strictly. Every rule a gate could get wrong is checked here rather than at
/// proving time, so a holder learns about a malformed rule before spending fifteen seconds on a
/// proof for it.
///
/// ```
/// use unstirred_core::parse_manifest;
///
/// let good = r#"{"version":"unstirred/v1","network":"test","context":"holders-room",
///                "window":{"h0":4505790,"h1":4506510},"min_zatoshis":"12500000"}"#;
/// assert!(parse_manifest(good).is_ok());
///
/// // An unknown field is a refusal, not something to ignore: a gate cannot smuggle a term in.
/// let extra = r#"{"version":"unstirred/v1","network":"test","context":"holders-room",
///                 "window":{"h0":4505790,"h1":4506510},"min_zatoshis":"12500000","fee":"1"}"#;
/// assert_eq!(parse_manifest(extra).unwrap_err().code, unstirred_core::Code::ManifestInvalid);
/// ```
pub fn parse_manifest(json: &str) -> Result<Manifest, Refusal> {
    let manifest: Manifest = serde_json::from_str(json)
        .map_err(|e| Refusal::new(Code::ManifestInvalid, format!("not a valid manifest: {e}")))?;
    require_supported_version(&manifest)?;
    require_valid_context(&manifest.context)?;
    require_fixed_threshold(&manifest)?;
    Ok(manifest)
}

fn require_supported_version(manifest: &Manifest) -> Result<(), Refusal> {
    if manifest.version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(Refusal::new(
            Code::ManifestInvalid,
            format!("unsupported version {:?}", manifest.version),
        ))
    }
}

fn require_valid_context(context: &str) -> Result<(), Refusal> {
    let mut chars = context.chars();
    let starts_alphanumeric = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    let rest_allowed = context
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if starts_alphanumeric && rest_allowed && context.len() <= MAX_CONTEXT_LEN {
        Ok(())
    } else {
        Err(Refusal::new(
            Code::ManifestInvalid,
            "context must be 1-64 chars of a-z, 0-9 and '-', starting with a letter or digit",
        ))
    }
}

fn require_fixed_threshold(manifest: &Manifest) -> Result<(), Refusal> {
    if manifest.min_zatoshis == REQUIRED_MIN_ZATOSHIS {
        Ok(())
    } else {
        Err(Refusal::new(
            Code::ManifestInvalid,
            "v1 supports only min_zatoshis = \"12500000\" (0.125 ZEC)",
        ))
    }
}

/// The JSON a gate owner should publish for one room, or a refusal naming what is wrong with the
/// values. The written text is only returned once [`parse_manifest`] has accepted it, so nothing
/// this function writes can be a manifest a verifier would reject.
///
/// ```
/// use unstirred_core::{build_manifest, parse_manifest, Network, Window};
///
/// let json = build_manifest(Network::Test, "holders-room", Window { h0: 4_505_790, h1: 4_506_510 })?;
/// assert_eq!(parse_manifest(&json)?.context, "holders-room");
///
/// // The window's own rules apply here too: h1 must be a multiple of ten.
/// assert!(build_manifest(Network::Test, "holders-room", Window { h0: 1, h1: 11 }).is_err());
/// # Ok::<(), unstirred_core::Refusal>(())
/// ```
pub fn build_manifest(network: Network, context: &str, window: Window) -> Result<String, Refusal> {
    require_valid_context(context)?;
    crate::window::check_window_shape(window)?;
    let manifest = Manifest {
        version: PROTOCOL_VERSION.to_owned(),
        network,
        context: context.to_owned(),
        window,
        min_zatoshis: REQUIRED_MIN_ZATOSHIS.to_owned(),
    };
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| Refusal::new(Code::ManifestInvalid, format!("the manifest could not be written: {e}")))?;
    parse_manifest(&json)?;
    Ok(json)
}
