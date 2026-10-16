#![doc = "The private nullifier lookup at h1. Every nullifier the prover asks about is queried \
through Valar's PIR client, which only ever puts it on the wire inside a YPIR ciphertext. There \
is no plaintext path: if the server cannot be reached the lookup refuses with PIR_UNAVAILABLE and \
the prover stops. The same oracle answers the delegation builder's questions about its padding \
slots, so one connection serves a whole proof and no nullifier is asked about twice."]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use pir_client::{PirClientBlocking, COMPILED_PIR_LAYOUT};
use unstirred_core::Network;
use voting_circuits::delegation::{ImtError, ImtProofData, ImtProvider};
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::{pallas, Fp};

use crate::error::ProverError;
use crate::imt::{circuit_proof, proves_absence, root_bytes};
use crate::nullifier::NoteNullifier;
use crate::snapshot::{describe, unavailable};
use crate::transport::HttpTransport;

/// The h1 nullifier set as the prover is allowed to ask about it: the delegation builder's
/// [`ImtProvider`], plus the typed reason the last question failed.
///
/// The builder's trait can only answer with [`ImtError`], a string. That is enough to stop a
/// proof but not enough to tell a holder what happened: a service that could not be reached is
/// `PIR_UNAVAILABLE` (inconclusive, retryable) and must never be read as "this note has moved".
pub trait NullifierSetAtH1: ImtProvider {
    /// Takes the typed error behind the most recent failed lookup, if there was one.
    fn last_failure(&self) -> Option<ProverError>;
}

/// One selected note's non-membership proof at h1, in the shape the delegation circuit takes.
/// `proven_unspent` is the out-of-circuit check of that same proof against the note's nullifier.
///
/// [`fmt::Debug`] shows neither the path nor the leaf bounds: those bracket the holder's
/// nullifier, and the circuit keeps them as private witnesses.
#[derive(Clone)]
pub struct NoteNonMembership {
    pub position: u64,
    pub proof: ImtProofData,
    pub proven_unspent: bool,
}

impl fmt::Debug for NoteNonMembership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoteNonMembership")
            .field("position", &self.position)
            .field("proven_unspent", &self.proven_unspent)
            .finish_non_exhaustive()
    }
}

/// Every selected note's lookup, against one h1 nullifier-set root.
#[derive(Debug, Clone)]
pub struct NullifierLookup {
    pub height: u32,
    pub root: [u8; 32],
    pub notes: Vec<NoteNonMembership>,
}

impl NullifierLookup {
    pub fn all_proven_unspent(&self) -> bool {
        self.notes.iter().all(|note| note.proven_unspent)
    }
}

/// A connection to one PIR server, pinned to one snapshot: the gate's h1 and the root that server
/// advertises for it. Every proof it returns is checked against that root before it is handed on.
pub struct PirNullifierOracle {
    client: PirClientBlocking,
    height: u32,
    root: [u8; 32],
    root_field: pallas::Base,
    answered: Mutex<BTreeMap<[u8; 32], ImtProofData>>,
    last_failure: Mutex<Option<ProverError>>,
}

impl fmt::Debug for PirNullifierOracle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PirNullifierOracle")
            .field("height", &self.height)
            .field("root", &hex::encode(self.root))
            .finish_non_exhaustive()
    }
}

impl PirNullifierOracle {
    /// Agrees a snapshot with the server before any query is sent: `GET /root` must describe this
    /// gate's network, pool, dataset version, h1 and tree layout, and must match `expected_root`
    /// when the gate pinned one.
    pub fn connect(
        pir_url: &str,
        network: Network,
        h1: u32,
        expected_root: Option<[u8; 32]>,
    ) -> Result<Self, ProverError> {
        let base = pir_url.trim_end_matches('/');
        let transport = Arc::new(HttpTransport::for_url(base)?);
        let dataset = describe(&transport, base, network, h1)?;
        if let Some(expected) = expected_root {
            require_root(expected, dataset.root)?;
        }
        let client = PirClientBlocking::with_transport(base, COMPILED_PIR_LAYOUT, transport)
            .map_err(|e| unavailable(format!("the PIR client could not start against {base}: {e}")))?;
        let root_field =
            Option::from(pallas::Base::from_repr(dataset.root)).ok_or_else(|| ProverError::PirRootNotCanonical {
                root: hex::encode(dataset.root),
            })?;
        Ok(Self {
            client,
            height: dataset.height,
            root: dataset.root,
            root_field,
            answered: Mutex::new(BTreeMap::new()),
            last_failure: Mutex::new(None),
        })
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// The root this oracle's answers authenticate to, as the hex a gate pins at h1. Named apart
    /// from [`ImtProvider::root`], which is the same root as a field element.
    pub fn root_bytes(&self) -> [u8; 32] {
        self.root
    }

    /// One private query, or the answer already held for that nullifier. A proof against any
    /// other root is refused: a lookup that does not speak about this snapshot proves nothing.
    pub fn proof_for(&self, nullifier: Fp) -> Result<ImtProofData, ProverError> {
        if let Some(held) = self.held(nullifier) {
            return Ok(held);
        }
        let returned = self
            .client
            .fetch_proof(nullifier)
            .map_err(|e| ProverError::NullifierProofUnobtainable(format!("{e}")))?;
        let proof = circuit_proof(&returned);
        let returned_root = root_bytes(&proof);
        if returned_root != self.root {
            return Err(ProverError::PirRootsDisagree {
                first: hex::encode(self.root),
                other: hex::encode(returned_root),
            });
        }
        self.remember(nullifier, &proof);
        Ok(proof)
    }

    /// Looks up one selected note, keeping the note's tree position next to the answer.
    pub fn look_up(&self, nullifier: &NoteNullifier) -> Result<NoteNonMembership, ProverError> {
        let proof = self.proof_for(nullifier.value()).map_err(|source| match source {
            ProverError::NullifierProofUnobtainable(detail) => ProverError::NullifierLookupFailed {
                position: nullifier.position(),
                detail,
            },
            other => other,
        })?;
        Ok(NoteNonMembership {
            position: nullifier.position(),
            proven_unspent: proves_absence(&proof, nullifier.value()),
            proof,
        })
    }

    fn held(&self, nullifier: Fp) -> Option<ImtProofData> {
        self.answered.lock().ok()?.get(&nullifier.to_repr()).cloned()
    }

    fn remember(&self, nullifier: Fp, proof: &ImtProofData) {
        if let Ok(mut answered) = self.answered.lock() {
            answered.insert(nullifier.to_repr(), proof.clone());
        }
    }

    fn record_failure(&self, failure: ProverError) -> ImtError {
        let message = failure.to_string();
        if let Ok(mut last) = self.last_failure.lock() {
            *last = Some(failure);
        }
        ImtError(message)
    }
}

impl ImtProvider for PirNullifierOracle {
    fn root(&self) -> pallas::Base {
        self.root_field
    }

    fn non_membership_proof(&self, nf: pallas::Base) -> Result<ImtProofData, ImtError> {
        self.proof_for(nf).map_err(|failure| self.record_failure(failure))
    }
}

impl NullifierSetAtH1 for PirNullifierOracle {
    fn last_failure(&self) -> Option<ProverError> {
        self.last_failure.lock().ok()?.take()
    }
}

/// Fetches and checks a non-membership proof for each nullifier at h1.
///
/// The server's advertised root is checked against `expected_root` before a single query is sent,
/// so a server holding the wrong snapshot is refused rather than queried. Every returned proof
/// must then authenticate to that same root. This blocks, and must not be called from inside an
/// async runtime.
pub fn look_up_nullifiers(
    pir_url: &str,
    network: Network,
    h1: u32,
    expected_root: Option<[u8; 32]>,
    nullifiers: &[NoteNullifier],
) -> Result<NullifierLookup, ProverError> {
    if nullifiers.is_empty() {
        return Err(ProverError::NoNullifiersToLookUp);
    }
    let oracle = PirNullifierOracle::connect(pir_url, network, h1, expected_root)?;
    let mut notes = Vec::with_capacity(nullifiers.len());
    for nullifier in nullifiers {
        notes.push(oracle.look_up(nullifier)?);
    }
    Ok(NullifierLookup {
        height: oracle.height(),
        root: oracle.root_bytes(),
        notes,
    })
}

/// A root the gate pinned at h1 and a root a server serves must be the same root, or nothing a
/// lookup returns means anything.
pub(crate) fn require_root(expected: [u8; 32], serving: [u8; 32]) -> Result<(), ProverError> {
    if expected == serving {
        Ok(())
    } else {
        Err(ProverError::PirRootMismatch {
            expected: hex::encode(expected),
            returned: hex::encode(serving),
        })
    }
}
