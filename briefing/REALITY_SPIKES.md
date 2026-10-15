# REALITY_SPIKES: Unstirred

Go/no-go deadline: 2026-10-17 evening. Record what actually happened, including failures.
Environment for offline spikes: sandbox, 1 CPU, 3 GB RAM, Rust 1.91.1 (Ubuntu `rustc-1.91`), voting-circuits 0.12.2, backend `lrz` (crates.io orchard 0.15.5).

## Results

| ID | Question | Result | Evidence |
|---|---|---|---|
| S1 | Valar's circuits build and their real delegation proof round-trip passes | **PASS** 2026-10-15. Release build 4m50s. Upstream `real_proof_roundtrip` green, including verification with freshly generated params (no prover state shared). | spikes/evidence/S1-upstream-roundtrip.log |
| S2 | Proof with nc_root pinned to H0 and nf_imt_root pinned to H1 verifies | **PASS** (synthetic tree + IMT, real K=12 proofs). About 10-15 s per proof on 1 core. | spikes/split_height_spike.rs, evidence/S2-S3-S6-results.log |
| S3 | Negative cases | **PASS**. Note created after H0: proof against its own later root is rejected when the verifier pins the H0 root. Note spent in (H0, H1]: no non-membership proof exists. Marker identical for the same coins in one context across different windows; different across contexts. | same |
| S6 | Spend-authority binding | **PASS**, with a key finding: a viewing-key holder CAN produce a valid proof alone. A RedPallas signature under rk over (statement, session key, challenge) fixes it: holder accepted, forged key rejected, replay to another session or stale challenge rejected. rk is rebuilt from the instance coordinates, never taken from the prover. | spikes/spend_authority_spike.rs, same log |
| S4 | nf-server syncs Ironwood nullifiers to H1 (multiple of 10), serves PIR, `verify-root` reproduces root | **OFFLINE PASS 2026-10-15; live part not run.** (a) Valar `imt_circuit_integration` 3/3: production-style sentinel IMT verifies in the delegation circuit. (b) `pir-client` unit tests 28/28, incl. `fetch_proof_local_verifies_for_known_ranges` and `process_tier1_and_build_produces_verifiable_proof` (only failure: doctest, because the sandbox lacks `rustdoc`). (c) `pir-test small`: synthetic 1,000-nullifier tree exported to tiers, 10/10 lookups return proofs that verify. Unconfirmed whether this mode runs the YPIR encryption itself or only the tier lookup. (d) `convert_pir_proof` is a field-for-field copy into the circuit type. Watch item: imt-tree 0.2.0 (circuit tests) vs 0.5.4 (server). Remaining: live testnet sync to H1 + `verify-root` (needs network). | evidence/S4-offline-*.log |
| S5 | Prover rebuilds each note's path against the H0 root | **OFFLINE PASS 2026-10-15; risk retired.** zcash_voting witness tests 24/24 incl. `generate_note_witnesses_uses_ironwood_tree_at_nu6_3`, wrong-root and stale-tree rejections. Code read: `zcash_client_sqlite 0.22 generate_ironwood_witnesses_at_historical_height` builds an in-memory ShardTree from the wallet's stored shards plus the caller-supplied frontier at H0 and inserts one checkpoint itself, so the wallet does NOT need to have retained a checkpoint at H0. Same crate cohort as the user's zcash-devtool wallet. Also: zcash_voting `test_real_delegation_proof` green (25 s). Remaining: run against the real wallet with TreeState at H0 = 4,505,790. | evidence/S5-offline-witness-tests.log, S1b-zcash_voting-real-delegation-proof.log |
| S7 | Network choice | **Leaning testnet, not confirmed.** Valar's sync supports testnet; whether the pinned crates (backend 0.24 / sqlite 0.22) sync NU7 testnet is confirmed by running S4/S5. Fallback: mainnet with a small amount. | runbook below |
| S8 | Real funds held across a real window | **FUNDED 2026-10-15.** Testnet wallet (zcash-devtool, account 56099e58-…, 'unstirred'), two Ironwood notes: note A 1.000 TAZ mined at 4,505,779 (tx 5b972dd8…94d4cd, Fauzec); note B 0.125 TAZ mined at 4,505,784 (tx ba1d9692…1d898c, Valar). Earliest valid H0: 4,505,779 for A alone, 4,505,784 for both. Recorded H0 = 4,505,790. Observed ~27 s blocks on NU7 testnet (5 blocks in ~134 s), so about 3,200-3,456 blocks per day. Coins must not move. | user wallet list-tx 2026-10-15 |

## Verdict so far
The cryptographic risks are retired: the split-height statement holds in real proofs, every negative case behaves, and the
one attack found (viewing-key-only proving) has a tested fix. What remains is integration with live chain data (S4, S5, S7, S8),
which uses Valar's existing, documented tooling. **GO for the planning phase; the first build milestone is the real-data path.**

## Runbook for the chain-dependent spikes (run on your machine)
Needs: Rust 1.91+, about 15 GB disk, network access to a testnet lightwalletd.

1. **Wallet with Ironwood testnet notes (S8).** Use a CLI wallet built on zcash_client_sqlite (e.g. zcash-devtool; first check its README for Ironwood / NU6.3 support). Create a testnet wallet, get a unified address, receive TAZ, sync. Then leave those coins untouched. Record the block height when they confirm: H0 must be at or after that.
2. **Nullifier set at H1 (S4).** In vote-nullifier-pir:
   `SVOTE_ZCASH_NETWORK=test LWD_URLS="<testnet lightwalletd>" SYNC_HEIGHT=<H1, multiple of 10> PIR_DATA_DIR=pir-data/test make sync`
   then run `nf-server verify-root` (see docs/verify-root.md) and record that both roots match.
3. **Coin record at H0 (S5).** Fetch the tree state at H0 (Zebra `z_gettreestate <H0>` or lightwalletd GetTreeState) and generate witnesses through zcash_voting's witness helper against your wallet DB.
4. **End to end.** Build one proof with (root@H0, imt@H1) and the S6 signature, then verify in a separate process. Save heights, roots and the result as evidence.

Paste the outputs (or errors) back into the chat and we record them here.
