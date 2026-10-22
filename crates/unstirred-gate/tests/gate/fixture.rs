//! A gate on a real socket, hosting two rooms over a synthetic wallet. **Synthetic:** the wallet,
//! its Ironwood tree and the nullifier set are built in this process, so the roots the gate pins
//! are this process's roots and the proofs are real K=12 proofs about them. The config is read by
//! the gate's own `load_config`, the rooms come from `unstirred gate init`'s own manifest builder,
//! and the challenge always comes from the running gate.
//!
//! Each proof costs seconds, so a test builds only the proofs its own behaviour needs.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use unstirred_core::{build_manifest, Manifest, Network, Window};
use unstirred_gate::{load_config, start, CollectedLog, GateClient, GateState, RunningGate};
use unstirred_prover::testing::{NoteA, SyntheticNullifierSet, SyntheticWallet, CHAIN_TIP, H0, H1};
use unstirred_prover::{eligible_notes, prove, witnesses_at, OsRng, ProofRequest, ReadOnlyWallet};
use unstirred_verifier::store::{Clock, Store};

use super::roots::FixedRoots;

pub type TestResult = Result<(), Box<dyn Error>>;

pub const HOLDERS_ROOM: &str = "holders-room";
pub const SECOND_ROOM: &str = "second-room";
pub const NOW: i64 = 1_700_000_000;

/// The gate's clock, stopped, so a challenge issued in a test is still open when the proof that
/// answers it is finished. Expiry itself is the store's own test (A07).
#[derive(Debug, Clone, Copy)]
pub struct StoppedClock;

impl Clock for StoppedClock {
    fn unix_seconds(&self) -> i64 {
        NOW
    }
}

pub struct Fixture {
    pub client: GateClient,
    pub log: CollectedLog,
    pub database: PathBuf,
    pub manifests: BTreeMap<String, Manifest>,
    wallet_directory: PathBuf,
    synthetic: SyntheticWallet,
    _gate: RunningGate,
    _directory: tempfile::TempDir,
}

impl Fixture {
    pub fn start() -> Result<Self, Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
        let wallet_directory = directory.path().join("wallet");
        std::fs::create_dir(&wallet_directory)?;
        synthetic.write_to(&wallet_directory)?;
        let nc_root_at_h0 = synthetic.tree_state_at(H0)?.root_bytes();
        let nf_imt_root_at_h1 = SyntheticNullifierSet::with_spent(&[]).root_bytes();

        let mut manifests = BTreeMap::new();
        for context in [HOLDERS_ROOM, SECOND_ROOM] {
            let json = build_manifest(Network::Test, context, Window { h0: H0, h1: H1 })?;
            std::fs::write(directory.path().join(format!("{context}.json")), &json)?;
            manifests.insert(context.to_owned(), unstirred_core::parse_manifest(&json)?);
        }
        let config_path = directory.path().join("gate.toml");
        std::fs::write(&config_path, config_toml(&hex::encode(nf_imt_root_at_h1)))?;
        let config = load_config(&config_path)?;
        let database = config.database.clone();
        let log = CollectedLog::default();
        let roots = FixedRoots {
            nc_root_at_h0,
            tip: CHAIN_TIP,
            sources: vec!["fake:first".to_owned(), "fake:second".to_owned()],
        };
        let gate = start(GateState::new(
            config,
            Arc::new(roots),
            Arc::new(StoppedClock),
            Arc::new(log.clone()),
        ))?;
        let client = GateClient::new(&gate.url())?;
        Ok(Self {
            client,
            log,
            database,
            manifests,
            wallet_directory,
            synthetic,
            _gate: gate,
            _directory: directory,
        })
    }

    /// A real proof of this wallet's held note for one room, answering one challenge. The gate is
    /// not involved: this is the holder's side, and the package is what a knock would post.
    pub fn prove_for(&self, room: &str, challenge: [u8; 32]) -> Result<String, Box<dyn Error>> {
        let manifest = self.manifests.get(room).ok_or("no such room in the fixture")?;
        let wallet = ReadOnlyWallet::open_with_params(&self.wallet_directory, self.synthetic.network())?;
        let account = self.synthetic.account()?;
        let authority = wallet.spend_authority_for(account, self.synthetic.ironwood_spending_key()?)?;
        let selected = eligible_notes(&wallet, account, manifest)?.selection;
        let witnesses = witnesses_at(&wallet, &self.synthetic.tree_state_at(H0)?, &selected.notes)?;
        let nullifier_set = SyntheticNullifierSet::with_spent(&[]);
        let proven = prove(
            &wallet,
            account,
            &authority,
            &ProofRequest {
                manifest,
                witnesses: &witnesses,
                notes_unspent_at: H1,
                expected_nf_imt_root: nullifier_set.root_bytes(),
                challenge,
            },
            &nullifier_set,
            &mut OsRng,
        )?;
        Ok(proven.package.to_json()?)
    }

    /// What the gate's own file holds, read directly, for the assertions a response cannot make.
    pub fn store(&self) -> Result<Store, Box<dyn Error>> {
        Ok(Store::open(&self.database)?)
    }
}

/// The two root-source specs are here only because a gate refuses to start without two. The roots
/// under test come from the injected authority, so neither URL is ever contacted.
fn config_toml(h1_root: &str) -> String {
    format!(
        r#"listen = "127.0.0.1:0"
database = "gate.sqlite"
root_sources = ["lwd:https://first.invalid:443", "zebra:http://second.invalid:18232"]
h1_root = "{h1_root}"

[[rooms]]
manifest = "{HOLDERS_ROOM}.json"

[[rooms]]
manifest = "{SECOND_ROOM}.json"
"#
    )
}
