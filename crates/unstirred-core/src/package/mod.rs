#![doc = "The proof package a holder hands a gate (PRD section 3.4). Both sides own this file: \
the prover writes a package, the verifier reads one, and neither invents a field name. The \
window, context and network in it are echoed for diagnostics only; the verifier takes those, and \
every root, from the manifest and its own root authority, never from here (PRD section 6)."]

mod instance;

pub use instance::PublicInstance;

use serde::{Deserialize, Serialize};
use voting_circuits::delegation::Instance;
use voting_crypto_deps::orchard::primitives::redpallas::{self, SpendAuth};

use crate::codes::{Code, Refusal};
use crate::manifest::{Manifest, Network, Window, PROTOCOL_VERSION};

const SIGNATURE_BYTES: usize = 64;
const SESSION_BYTES: usize = 32;
const CHALLENGE_BYTES: usize = 32;

/// One holder's answer to one challenge at one gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofPackage {
    pub version: String,
    pub context: String,
    pub network: Network,
    pub window: Window,
    pub instance: PublicInstance,
    pub proof: String,
    pub session_pubkey: String,
    pub challenge: String,
    pub binding_signature: String,
}

impl ProofPackage {
    pub fn new(
        manifest: &Manifest,
        instance: &Instance,
        proof: &[u8],
        session_pubkey: &[u8; SESSION_BYTES],
        challenge: &[u8; CHALLENGE_BYTES],
        binding_signature: &redpallas::Signature<SpendAuth>,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_owned(),
            context: manifest.context.clone(),
            network: manifest.network,
            window: manifest.window,
            instance: PublicInstance::of(instance),
            proof: hex::encode(proof),
            session_pubkey: hex::encode(session_pubkey),
            challenge: hex::encode(challenge),
            binding_signature: hex::encode(<[u8; SIGNATURE_BYTES]>::from(binding_signature)),
        }
    }

    /// Reads a package strictly: an unknown field, a missing field or a wrong version is refused
    /// rather than ignored.
    pub fn parse(json: &str) -> Result<Self, Refusal> {
        let package: Self = serde_json::from_str(json)
            .map_err(|e| Refusal::new(Code::ProofInvalid, format!("not a valid proof package: {e}")))?;
        if package.version == PROTOCOL_VERSION {
            Ok(package)
        } else {
            Err(malformed(
                "version",
                &format!("{:?}, not {PROTOCOL_VERSION}", package.version),
            ))
        }
    }

    pub fn to_json(&self) -> Result<String, Refusal> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Refusal::new(Code::ProofInvalid, format!("the package could not be written: {e}")))
    }

    pub fn instance(&self) -> Result<Instance, Refusal> {
        self.instance.decode()
    }

    pub fn proof_bytes(&self) -> Result<Vec<u8>, Refusal> {
        hex::decode(&self.proof).map_err(|_| malformed("proof", "not hex"))
    }

    pub fn session_pubkey(&self) -> Result<[u8; SESSION_BYTES], Refusal> {
        bytes_from_hex("session_pubkey", &self.session_pubkey)
    }

    pub fn challenge(&self) -> Result<[u8; CHALLENGE_BYTES], Refusal> {
        bytes_from_hex("challenge", &self.challenge)
    }

    pub fn binding_signature(&self) -> Result<redpallas::Signature<SpendAuth>, Refusal> {
        Ok(redpallas::Signature::from(bytes_from_hex::<SIGNATURE_BYTES>(
            "binding_signature",
            &self.binding_signature,
        )?))
    }
}

fn bytes_from_hex<const N: usize>(name: &str, text: &str) -> Result<[u8; N], Refusal> {
    let bytes = hex::decode(text).map_err(|_| malformed(name, "not hex"))?;
    <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| malformed(name, &format!("not {N} bytes")))
}

/// A package a verifier cannot even read is not a valid proof, so a malformed field is
/// `PROOF_INVALID` rather than a code of its own.
fn malformed(field: &str, detail: &str) -> Refusal {
    Refusal::new(Code::ProofInvalid, format!("package field {field} is {detail}"))
}
