#![doc = "One connection to one PIR server, pinned to one snapshot. Every nullifier leaves this \
process inside a YPIR ciphertext, every answer is checked against the snapshot's root, and an answer \
already held is never asked for twice."]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use pir_client::{PirClientBlocking, COMPILED_PIR_LAYOUT};
use unstirred_core::Network;
use voting_circuits::delegation::{ImtError, ImtProofData, ImtProvider};
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::{pallas, Fp};

use super::{require_root, NoteNonMembership, NullifierSetAtH1};
use crate::error::ProverError;
use crate::imt::{circuit_proof, proves_absence, root_bytes};
use crate::nullifier::NoteNullifier;
use crate::snapshot::{describe, unavailable};
use crate::transport::HttpTransport;

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

    /// The height of the snapshot this oracle answers about, which must be the gate's h1.
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
