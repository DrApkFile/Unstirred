//! The synthetic wallet: two Ironwood notes mined at activation, scanned through `H1`, with the
//! Ironwood note commitment tree snapshotted at the end of every block.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

use unstirred_core::{parse_manifest, Manifest};
use voting_crypto_deps::orchard::keys::SpendingKey;
use zcash_client_backend::data_api::testing::orchard::OrchardPoolTester;
use zcash_client_backend::data_api::testing::pool::ShieldedPoolTester;
use zcash_client_backend::data_api::testing::sapling::SaplingPoolTester;
use zcash_client_backend::data_api::testing::{AddressType, IronwoodFvk, TestBuilder, TestState};
use zcash_client_backend::data_api::{Account, WalletWrite};
use zcash_client_sqlite::testing::db::{TestDb, TestDbFactory};
use zcash_client_sqlite::testing::BlockCache;
use zcash_client_sqlite::AccountUuid;
use zcash_primitives::block::BlockHash;
use zcash_protocol::consensus::BlockHeight;
use zcash_protocol::local_consensus::LocalNetwork;
use zcash_protocol::value::Zatoshis;

use crate::{IronwoodFrontier, IronwoodTreeState, WALLET_DATABASE_FILE};

pub const ACTIVATION: u32 = 100_000;
pub const NOTE_A_ZATOSHIS: u64 = 100_000_000;
pub const NOTE_B_ZATOSHIS: u64 = 12_500_000;

/// Note A is mined at `ACTIVATION`, note B one block later.
pub const NOTE_A_HEIGHT: u32 = ACTIVATION;
pub const NOTE_B_HEIGHT: u32 = ACTIVATION + 1;

/// `h0` sits above both notes, `h1` is a multiple of ten, and the recorded tip is far enough above
/// `h1` for the window rules in `unstirred-core` to accept it.
pub const H0: u32 = ACTIVATION + 5;
pub const H1: u32 = ACTIVATION + 20;
pub const CHAIN_TIP: u32 = H1 + 200;

/// The two notes occupy the first two blocks above activation; the rest are empty.
const FIRST_EMPTY_BLOCK: u32 = ACTIVATION + 2;
const BLOCKS_TO_SCAN: usize = (H1 - ACTIVATION) as usize + 1;

/// Inside the window: above `H0`, at or below `H1`.
pub const SPEND_HEIGHT: u32 = ACTIVATION + 10;

/// What happens to the larger note after it is received.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteA {
    StaysPut,
    SpentInsideTheWindow,
}

/// A wallet with two Ironwood notes, scanned through `H1`, with the chain tip recorded at
/// `CHAIN_TIP`, plus the Ironwood note commitment tree as it stood at the end of every block.
/// Those snapshots are what `GetTreeState` would return for those heights.
pub struct SyntheticWallet {
    state: TestState<BlockCache, TestDb, LocalNetwork>,
    ironwood_trees: BTreeMap<u32, IronwoodFrontier>,
}

impl std::fmt::Debug for SyntheticWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SyntheticWallet")
    }
}

impl SyntheticWallet {
    pub fn build(note_a: NoteA) -> Result<Self, Box<dyn Error>> {
        let activation = BlockHeight::from_u32(ACTIVATION);
        let network = LocalNetwork {
            nu6: Some(activation),
            nu6_1: Some(activation),
            nu6_2: Some(activation),
            nu6_3: Some(activation),
            ..TestBuilder::<(), ()>::DEFAULT_NETWORK
        };
        let mut state = TestBuilder::new()
            .with_network(network)
            .with_data_store_factory(TestDbFactory::default())
            .with_block_cache(BlockCache::new())
            .with_account_from_sapling_activation(BlockHash([0; 32]))
            .build();
        let mut ironwood_trees = BTreeMap::new();

        let ironwood = IronwoodFvk(OrchardPoolTester::test_account_fvk(&state));
        let value_a = Zatoshis::const_from_u64(NOTE_A_ZATOSHIS);
        let (_, _, nullifier_a) = state.generate_next_block(&ironwood, AddressType::DefaultExternal, value_a);
        snapshot(&state, &mut ironwood_trees);
        state.generate_next_block(
            &ironwood,
            AddressType::DefaultExternal,
            Zatoshis::const_from_u64(NOTE_B_ZATOSHIS),
        );
        snapshot(&state, &mut ironwood_trees);

        let spend_at = match note_a {
            NoteA::StaysPut => H1 + 1,
            NoteA::SpentInsideTheWindow => SPEND_HEIGHT,
        };
        for height in FIRST_EMPTY_BLOCK..=H1 {
            if height == spend_at {
                let stranger = SaplingPoolTester::sk_to_fvk(&SaplingPoolTester::sk(&[0xf5; 32]));
                state.generate_next_block_spending(
                    &ironwood,
                    (nullifier_a, value_a),
                    SaplingPoolTester::fvk_default_address(&stranger),
                    Zatoshis::const_from_u64(NOTE_A_ZATOSHIS / 2),
                );
            } else {
                state.generate_empty_block();
            }
            snapshot(&state, &mut ironwood_trees);
        }
        state.scan_cached_blocks(activation, BLOCKS_TO_SCAN);
        state.wallet_mut().update_chain_tip(BlockHeight::from_u32(CHAIN_TIP))?;
        Ok(Self { state, ironwood_trees })
    }

    pub fn account(&self) -> Result<AccountUuid, Box<dyn Error>> {
        Ok(self
            .state
            .test_account()
            .ok_or("the test harness built no account")?
            .account()
            .id())
    }

    /// The test account's Ironwood spending key, as the harness derived it from its own seed.
    /// That seed is 32 zero bytes, not a mnemonic, so this is the one way a test can hold the
    /// spending authority for a harness wallet.
    pub fn ironwood_spending_key(&self) -> Result<SpendingKey, Box<dyn Error>> {
        Ok(*self
            .state
            .test_account()
            .ok_or("the test harness built no account")?
            .usk()
            .orchard())
    }

    pub fn network(&self) -> LocalNetwork {
        *self.state.network()
    }

    /// Copies the harness's wallet to a file, so it can be reopened the way the CLI opens the
    /// holder's wallet: a separate, read-only connection.
    pub fn write_to(&self, dir: &Path) -> Result<(), Box<dyn Error>> {
        let database = dir.join(WALLET_DATABASE_FILE);
        self.state
            .wallet()
            .conn()
            .execute(&format!("VACUUM INTO '{}'", database.display()), [])?;
        Ok(())
    }

    /// The Ironwood tree as it stood at the end of `height`, the way `GetTreeState` reports it.
    pub fn tree_state_at(&self, height: u32) -> Result<IronwoodTreeState, Box<dyn Error>> {
        let tree = self
            .ironwood_trees
            .get(&height)
            .ok_or_else(|| format!("no Ironwood tree snapshot was taken at height {height}"))?;
        Ok(IronwoodTreeState::from_parts(height, tree.clone()))
    }
}

fn snapshot(state: &TestState<BlockCache, TestDb, LocalNetwork>, trees: &mut BTreeMap<u32, IronwoodFrontier>) {
    if let Some(block) = state.latest_cached_block() {
        trees.insert(
            u32::from(block.height()),
            block.chain_state().final_ironwood_tree().clone(),
        );
    }
}

pub fn manifest(h0: u32, h1: u32) -> Result<Manifest, Box<dyn Error>> {
    let json = format!(
        r#"{{"version":"unstirred/v1","network":"test","context":"holders-room",
             "window":{{"h0":{h0},"h1":{h1}}},"min_zatoshis":"12500000"}}"#
    );
    Ok(parse_manifest(&json)?)
}
