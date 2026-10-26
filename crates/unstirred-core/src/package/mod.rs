#![doc = "The proof package a holder hands a gate (PRD section 3.4). Both sides own this file: \
the prover writes a package, the verifier reads one, and neither invents a field name. The \
window, context and network in it are echoed for diagnostics only; the verifier takes those, and \
every root, from the manifest and its own root authority, never from here (PRD section 6)."]

mod instance;

/// The circuit's public inputs, as the package carries them.
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
    /// Always `unstirred/v1`; any other value is refused at parse time.
    pub version: String,
    /// The context the holder believes they are proving for. **Echoed for diagnostics only** —
    /// the verifier takes the context from the manifest and compares the instance's
    /// `vote_round_id` to it.
    pub context: String,
    /// The network, echoed. Not read as authority, for the same reason.
    pub network: Network,
    /// The window, echoed. Not read as authority: the verifier uses the gate's own.
    pub window: Window,
    /// The circuit's fourteen public inputs. This is the part that is actually checked.
    pub instance: PublicInstance,
    /// The Halo2 proof, hex.
    pub proof: String,
    /// The per-proof session public key, 32 bytes of hex. It has no private half in v1.
    pub session_pubkey: String,
    /// The gate's challenge this package answers, 32 bytes of hex.
    pub challenge: String,
    /// A RedPallas signature over the binding message, under `rk`, 64 bytes of hex. This is what
    /// stops a viewing-key holder proving as someone else.
    pub binding_signature: String,
}

impl ProofPackage {
    /// Assembles a package from everything the prover has just produced. The context, network and
    /// window are copied from the manifest, so a package can never claim a rule it was not made
    /// against.
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
    ///
    /// ```
    /// use unstirred_core::{Code, ProofPackage};
    ///
    /// // Anything a verifier cannot fully read is PROOF_INVALID — there is no partial package.
    /// let refusal = ProofPackage::parse("{}").unwrap_err();
    /// assert_eq!(refusal.code, Code::ProofInvalid);
    /// ```
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

    /// The package as the JSON a holder hands over. Pretty-printed, because people read these.
    pub fn to_json(&self) -> Result<String, Refusal> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Refusal::new(Code::ProofInvalid, format!("the package could not be written: {e}")))
    }

    /// The circuit instance these public inputs decode to, or a refusal if any of them is not a
    /// canonical field element.
    pub fn instance(&self) -> Result<Instance, Refusal> {
        self.instance.decode()
    }

    /// The proof as bytes.
    pub fn proof_bytes(&self) -> Result<Vec<u8>, Refusal> {
        hex::decode(&self.proof).map_err(|_| malformed("proof", "not hex"))
    }

    /// The session public key as 32 bytes.
    pub fn session_pubkey(&self) -> Result<[u8; SESSION_BYTES], Refusal> {
        bytes_from_hex("session_pubkey", &self.session_pubkey)
    }

    /// The challenge this package answers, as 32 bytes. The gate looks this up before it spends
    /// anything on the signature or the proof.
    pub fn challenge(&self) -> Result<[u8; CHALLENGE_BYTES], Refusal> {
        bytes_from_hex("challenge", &self.challenge)
    }

    /// The binding signature, ready to verify under `rk`.
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
