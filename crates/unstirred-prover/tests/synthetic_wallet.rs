//! Synthetic, not real chain data: a local test network, fake compact blocks and Ironwood notes
//! created by the `zcash_client_sqlite` test harness. These tests prove the wallet read path, not
//! anything about Zcash testnet. No real-testnet run is recorded yet (M1 step 2).

use std::error::Error;
use std::path::Path;

use unstirred_core::{parse_manifest, Code, Manifest};
use unstirred_prover::{eligible_notes, ProverError, ReadOnlyWallet, WALLET_DATABASE_FILE};
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

type TestResult = Result<(), Box<dyn Error>>;

const ACTIVATION: u32 = 100_000;
const NOTE_A_ZATOSHIS: u64 = 100_000_000;
const NOTE_B_ZATOSHIS: u64 = 12_500_000;

/// `h0` sits above both notes, `h1` is a multiple of ten, and the recorded tip is far enough above
/// `h1` for the window rules in `unstirred-core` to accept it.
const H0: u32 = ACTIVATION + 5;
const H1: u32 = ACTIVATION + 20;
const CHAIN_TIP: u32 = H1 + 200;

/// The two notes occupy the first two blocks above activation; the rest are empty.
const FIRST_EMPTY_BLOCK: u32 = ACTIVATION + 2;
const BLOCKS_TO_SCAN: usize = (H1 - ACTIVATION) as usize + 1;

/// Inside the window: above `H0`, at or below `H1`.
const SPEND_HEIGHT: u32 = ACTIVATION + 10;

/// What happens to the larger note after it is received.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteA {
    StaysPut,
    SpentInsideTheWindow,
}

fn manifest(h0: u32, h1: u32) -> Result<Manifest, Box<dyn Error>> {
    let json = format!(
        r#"{{"version":"unstirred/v1","network":"test","context":"holders-room",
             "window":{{"h0":{h0},"h1":{h1}}},"min_zatoshis":"12500000"}}"#
    );
    Ok(parse_manifest(&json)?)
}

/// A wallet with two Ironwood notes mined just after activation, scanned through `H1`, with the
/// chain tip recorded at `CHAIN_TIP`.
fn two_notes(note_a: NoteA) -> TestState<BlockCache, TestDb, LocalNetwork> {
    let activation = BlockHeight::from_u32(ACTIVATION);
    let network = LocalNetwork {
        nu6: Some(activation),
        nu6_1: Some(activation),
        nu6_2: Some(activation),
        nu6_3: Some(activation),
        ..TestBuilder::<(), ()>::DEFAULT_NETWORK
    };
    let mut st = TestBuilder::new()
        .with_network(network)
        .with_data_store_factory(TestDbFactory::default())
        .with_block_cache(BlockCache::new())
        .with_account_from_sapling_activation(BlockHash([0; 32]))
        .build();

    let ironwood = IronwoodFvk(OrchardPoolTester::test_account_fvk(&st));
    let value_a = Zatoshis::const_from_u64(NOTE_A_ZATOSHIS);
    let (_, _, nullifier_a) = st.generate_next_block(&ironwood, AddressType::DefaultExternal, value_a);
    st.generate_next_block(
        &ironwood,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(NOTE_B_ZATOSHIS),
    );
    match note_a {
        NoteA::StaysPut => {
            for _ in FIRST_EMPTY_BLOCK..=H1 {
                st.generate_empty_block();
            }
        }
        NoteA::SpentInsideTheWindow => {
            for _ in FIRST_EMPTY_BLOCK..SPEND_HEIGHT {
                st.generate_empty_block();
            }
            let stranger = SaplingPoolTester::sk_to_fvk(&SaplingPoolTester::sk(&[0xf5; 32]));
            st.generate_next_block_spending(
                &ironwood,
                (nullifier_a, value_a),
                SaplingPoolTester::fvk_default_address(&stranger),
                Zatoshis::const_from_u64(NOTE_A_ZATOSHIS / 2),
            );
            for _ in (SPEND_HEIGHT + 1)..=H1 {
                st.generate_empty_block();
            }
        }
    }
    st.scan_cached_blocks(activation, BLOCKS_TO_SCAN);
    st
}

/// Copies the harness's wallet to a file, so it can be reopened the way the CLI opens the
/// holder's wallet: a separate, read-only connection.
fn as_wallet_directory(st: &TestState<BlockCache, TestDb, LocalNetwork>, dir: &Path) -> TestResult {
    let database = dir.join(WALLET_DATABASE_FILE);
    st.wallet()
        .conn()
        .execute(&format!("VACUUM INTO '{}'", database.display()), [])?;
    Ok(())
}

fn account_of(st: &TestState<BlockCache, TestDb, LocalNetwork>) -> Result<AccountUuid, Box<dyn Error>> {
    Ok(st
        .test_account()
        .ok_or("the test harness built no account")?
        .account()
        .id())
}

fn refusal_code(error: ProverError) -> Option<Code> {
    match error {
        ProverError::Refused(refusal) => Some(refusal.code),
        _ => None,
    }
}

#[test]
fn a17_lists_the_ironwood_notes_held_across_the_window_with_value_and_mined_height() -> TestResult {
    let mut st = two_notes(NoteA::StaysPut);
    st.wallet_mut().update_chain_tip(BlockHeight::from_u32(CHAIN_TIP))?;
    let account = account_of(&st)?;
    let dir = tempfile::tempdir()?;
    as_wallet_directory(&st, dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), *st.network())?;
    let found = eligible_notes(&wallet, account, &manifest(H0, H1)?)?;

    let mut values: Vec<u64> = found
        .notes
        .held_through_h1
        .iter()
        .map(|note| note.value_zatoshis)
        .collect();
    values.sort_unstable();
    assert_eq!(values, vec![NOTE_B_ZATOSHIS, NOTE_A_ZATOSHIS]);
    assert!(
        found
            .notes
            .held_through_h1
            .iter()
            .all(|note| note.mined_height >= ACTIVATION && note.mined_height <= H0),
        "{:?}",
        found.notes.held_through_h1
    );
    assert_eq!(found.selection.notes.len(), 1, "{:?}", found.selection);
    assert_eq!(found.selection.total_zatoshis, u128::from(NOTE_A_ZATOSHIS));
    assert_eq!(found.chain_tip, CHAIN_TIP);
    assert_eq!(found.fully_scanned, H1);
    Ok(())
}

#[test]
fn a03_notes_mined_after_h0_are_refused_with_coins_too_new() -> TestResult {
    let mut st = two_notes(NoteA::StaysPut);
    st.wallet_mut().update_chain_tip(BlockHeight::from_u32(CHAIN_TIP))?;
    let account = account_of(&st)?;
    let dir = tempfile::tempdir()?;
    as_wallet_directory(&st, dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), *st.network())?;
    let before_any_note = manifest(ACTIVATION - 1, H1)?;
    let refused = eligible_notes(&wallet, account, &before_any_note);

    assert_eq!(
        refused.err().and_then(refusal_code),
        Some(Code::CoinsTooNew),
        "an h0 below every note's mined height must be COINS_TOO_NEW"
    );
    Ok(())
}

#[test]
fn a18_a_wallet_scanned_short_of_h1_is_refused_before_any_note_is_read() -> TestResult {
    let mut st = two_notes(NoteA::StaysPut);
    st.wallet_mut().update_chain_tip(BlockHeight::from_u32(CHAIN_TIP))?;
    let account = account_of(&st)?;
    let dir = tempfile::tempdir()?;
    as_wallet_directory(&st, dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), *st.network())?;
    let past_the_scan = manifest(H0, H1 + 10)?;
    let refused = eligible_notes(&wallet, account, &past_the_scan);

    assert_eq!(refused.err().and_then(refusal_code), Some(Code::WalletNotSyncedPastH1));
    Ok(())
}

#[test]
fn a04_a_note_spent_inside_the_window_drops_out_of_the_held_set() -> TestResult {
    let mut st = two_notes(NoteA::SpentInsideTheWindow);
    st.wallet_mut().update_chain_tip(BlockHeight::from_u32(CHAIN_TIP))?;
    let account = account_of(&st)?;
    let dir = tempfile::tempdir()?;
    as_wallet_directory(&st, dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), *st.network())?;
    let found = eligible_notes(&wallet, account, &manifest(H0, H1)?)?;

    assert!(
        found
            .notes
            .unspent_at_h0
            .iter()
            .any(|note| note.value_zatoshis == NOTE_A_ZATOSHIS),
        "the spent note was still unspent at h0: {:?}",
        found.notes.unspent_at_h0
    );
    assert_eq!(
        found
            .notes
            .held_through_h1
            .iter()
            .map(|note| note.value_zatoshis)
            .collect::<Vec<u64>>(),
        vec![NOTE_B_ZATOSHIS],
        "only the note that never moved may count as held"
    );
    assert_eq!(found.selection.total_zatoshis, u128::from(NOTE_B_ZATOSHIS));
    Ok(())
}
