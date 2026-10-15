use serde::{Deserialize, Serialize};

use crate::codes::{Code, Refusal};

pub const PROTOCOL_VERSION: &str = "unstirred/v1";
pub const REQUIRED_MIN_ZATOSHIS: &str = "12500000";
const MAX_CONTEXT_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Test,
    Main,
}

impl Network {
    pub const fn domain_byte(self) -> u8 {
        match self {
            Network::Test => b't',
            Network::Main => b'm',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub h0: u32,
    pub h1: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: String,
    pub network: Network,
    pub context: String,
    pub window: Window,
    pub min_zatoshis: String,
}

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
