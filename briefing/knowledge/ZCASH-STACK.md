# Zcash: complete build briefing

**Researched:** 2026-10-14
**Scope:** https://github.com/zcash/librustzcash and https://github.com/ZcashFoundation/zebra,
plus the specs, daemons and services they depend on.
**Method:** primary sources only — ZIP texts from `zcash/zips`, per-crate `CHANGELOG.md`
files, Zebra book sources, GitHub release notes, and the crates.io JSON API. Secondary
sources were used for orientation and are labelled as such wherever they appear.

---

## Contents

1. [Read this first](#1-read-this-first)
2. [Confidence conventions and sources](#2-confidence-conventions-and-sources)
3. [Network state and upgrades](#3-network-state-and-upgrades)
4. [librustzcash](#4-librustzcash)
5. [Zebra](#5-zebra)
6. [The surrounding stack: Zallet, Zaino, lightwalletd, Z3](#6-the-surrounding-stack)
7. [Wallet rules after Ironwood: ZIP 326 and ZIP 318](#7-wallet-rules-after-ironwood)
8. [Gotchas and traps](#8-gotchas-and-traps)
9. [Build paths and open ground](#9-build-paths-and-open-ground)
10. [What to re-verify, and open questions](#10-what-to-re-verify-and-open-questions)
11. [Source index](#11-source-index)

---

## 1. Read this first

**Almost everything in training data, tutorials and blog posts about Zcash is now out of
date.** Three structural changes landed between May and October 2026. Any material older
than **August 2026** is wrong on at least one of them:

1. **`zcashd` is dead.** It reached its automatic end-of-support halt on **2026-07-18 at
   block 3,417,100**, and the repository is archived read-only. The node is **Zebra**;
   the wallet is **Zallet**. There is no `zcashd` fallback.

2. **The Orchard pool is sealed.** NU6.3 ("Ironwood") activated **2026-07-28 at mainnet
   block 3,428,143**, following a soundness flaw found in Orchard in late May 2026. New
   value cannot enter Orchard. Shielded funds now live in the new **Ironwood** pool, and
   existing Orchard funds leave through a public-amount turnstile (ZIP 318).
   `zcash_client_backend` **rejects proposals that pay the Orchard pool** once Ironwood
   is active.

3. **NU7 is weeks away.** It is live on testnet (block 4,465,026, 2026-10-04) and cuts
   block time from 75s to 25s. The mainnet go/no-go decision is **2026-10-20**, targeting
   activation **2026-11-05**. No mainnet height is set yet.

If a build runs across early November, point 3 is a scheduling risk, not trivia.

---

## 2. Confidence conventions and sources

Every claim below carries one of three labels:

- **Verified** — the primary source was fetched in this session (ZIP text, repo file,
  CHANGELOG, crates.io API, release notes). URL cited in §11.
- **Reported** — consistent across multiple secondary sources, but the primary text
  could not be reached. Probably right; confirm before depending on it.
- **Unverified** — single source, or sources conflict. Do not build on it.

### Network constraint worth knowing

`zips.z.cash`, `docs.rs` and `zebra.zfnd.org` **do not resolve** from the research
environment used here (network policy, `ENOTFOUND`). Everything was read instead from
`raw.githubusercontent.com`, `github.com`, `crates.io` and `lib.rs`. The ZIP texts in
the `zcash/zips` GitHub repo are the same documents `zips.z.cash` renders, so nothing
was lost — but if you re-verify from a less restricted machine, prefer the rendered
sites.

---

## 3. Network state and upgrades

### 3.1 What the network is now

Zcash is a **Zebra-only network** running these value pools: transparent, Sprout
(legacy, effectively dead), Sapling, Orchard (**sealed** as of NU6.3) and **Ironwood**
(the live shielded pool). Current consensus rules are NU6.3. NU7 is live on testnet and
awaiting a mainnet decision.

### 3.2 Upgrade timeline

| Upgrade | Mainnet height | Testnet height | Date | Confidence |
|---|---|---|---|---|
| NU6.1 | 3,146,400 | 3,536,500 | 2025-11-24 | Reported (ZIP 255) |
| NU6.2 | ~3,363,426 (soft fork) | — | ~2026-06-02 | **Unverified — sources conflict** |
| **NU6.3 "Ironwood"** | **3,428,143** | **4,134,000** | 2026-07-28 | **Verified** (ZIP 258) |
| NU7 | not set | 4,465,026 | testnet 2026-10-04; mainnet target 2026-11-05 | Reported |

**NU6.3 consensus branch ID: `0x37A5165B`.** Minimum network protocol version **170160**
on both mainnet and testnet. (Verified, ZIP 258.)

On NU6.2: a GitHub release note ties a soft fork to mainnet block 3,363,426 (~02:00 UTC
2026-06-02) with a related Orchard action-disabling height of 3,363,366. A ZecHub wiki
PR describes it as an emergency fix to the Orchard circuit. These are a proposed wiki
edit and a release note, not an authoritative record — treat the NU6.2 numbers as
unverified.

### 3.3 ZIP statuses lag reality

ZIP 258 is still marked **Status: Draft** although NU6.3 activated nearly three months
ago. Same pattern elsewhere: ZIP 229 (Draft), ZIP 318 (Draft), ZIP 326 (Draft), ZIP 2005
(Proposed). **Do not infer "not live" from "Draft".**

Conversely, **ZIP 230 is genuinely Withdrawn** — its page is now titled "Withdrawn
Version 6 Transaction Format". The v6 format is **ZIP 229**.

### 3.4 What NU6.3 bundles (Verified, ZIP 258)

**Primary sources in the upgrade:**
- ZIP 200 — Network Upgrade Mechanism
- **ZIP 229 — Version 6 Transaction Format**
- **ZIP 2005 — Ironwood Quantum Recoverability** (activates at NU6.3)
- **ZIP 2006 — Restricting Transfers into the Orchard Pool**

**Updated by the upgrade:**
- ZIP 209 — Prohibit Out-of-Range Chain Value Pool Balances
- ZIP 213 — Shielded Coinbase
- ZIP 221 — FlyClient history tree changes (new Ironwood metadata fields)
- ZIP 317 — Proportional Fee Mechanism

**Wallet considerations:**
- **ZIP 318 — Orchard to Ironwood Migration**
- **ZIP 326 — NU6.3 Consequences for Wallets**

ZIP 201 (peer management and handshake) and ZIP 204 (protocol version assignment) also
apply.

> **Secondary sources miscite these constantly.** Articles variously credit ZIP 209,
> ZIP 230 or ZIP 318 with "the turnstile rules". Deployment is **258**; v6 format is
> **229**; quantum recoverability is **2005**; Orchard restriction is **2006**;
> migration is **318**; wallet consequences are **326**.

### 3.5 What Ironwood actually is

- **Not a new proof system.** Ironwood is a second value pool *of the Orchard protocol*,
  reusing the Orchard Action encoding and the Halo 2 proof system, with its **own note
  commitment tree, anchor, and chain value pool balance**. (Verified, ZIP 229.)

- **Its purpose is quantum *recoverability*, not quantum security.** (Verified, ZIP 2005.)
  - Ironwood notes use note plaintext **lead byte `0x03`**; Orchard uses `0x02`.
  - In Orchard, commitment randomness (`rcm`) derives from the note's random seed and
    nullifier-related values. In Ironwood, the derivation **also includes every note
    field** — recipient diversified generator and public key, value, rho, psi — which
    makes the note commitment **binding against a quantum adversary**, provided the
    recovery process checks that derivation.
  - Incoming-viewing-key derivation also changes. Keys can come from a legacy spending
    key, or — for FROST and some hardware wallets — from a new **quantum spending key
    (`qsk`)** and **quantum intermediate key (`qk`)**.
  - **This does not make Zcash post-quantum today.** It is groundwork for a possible
    future **Recovery Protocol**: a shielded protocol, sketched only in outline, that
    would let users spend Ironwood notes if discrete-log assumptions break. A spend
    would require a RedDSA signature on the spend validating key plus proof of knowledge
    of `qsk`. It assumes no particular post-quantum proof system or hash.
  - **Funds left in Sprout, Sapling or Orchard would not be recoverable.** That is the
    stated reason to migrate.

### 3.6 The v6 transaction format (Verified, ZIP 229)

A v6 transaction adds a **fifth bundle** on top of transparent / Sapling / Orchard.
Version group ID **`0xD884B698`**.

**Field layout:**
1. **Common:** `header`, `nVersionGroupId`, `nConsensusBranchId`, `lock_time`,
   `nExpiryHeight`
2. **Transparent:** `tx_in`, `tx_out`, with counts
3. **Sapling:** spend and output descriptions, `valueBalanceSapling`, `anchorSapling`,
   proofs, spend auth signatures, `bindingSigSapling`
4. **Orchard:** `nActionsOrchard`, `vActionsOrchard`, `flagsOrchard`,
   `valueBalanceOrchard`, `anchorOrchard`, `sizeProofsOrchard`, `proofsOrchard`,
   `vSpendAuthSigsOrchard`, `bindingSigOrchard`
5. **Ironwood (new):** `nActionsIronwood`, `vActionsIronwood`, `flagsIronwood`,
   `valueBalanceIronwood`, `anchorIronwood`, `sizeProofsIronwood`, `proofsIronwood`,
   `vSpendAuthSigsIronwood`, `bindingSigIronwood`

Several fields are present only when their component has actions. The Orchard and
Ironwood flag bytes share the same layout, including an `enableCrossAddress` bit.

**Two consensus changes worth internalising:**

- **Anchors became authorizing data in v6.** Sapling, Orchard and Ironwood anchors are
  no longer committed to by the txid, so a transaction can be **pre-authorized and have
  its anchor and proofs refreshed later without the txid changing**. This is precisely
  what makes the ZIP 318 pre-signed migration schedule possible.
- **Orchard-pool actions are banned in coinbase transactions.**

ZIP 229 also covers txid, auth digest, signature digest and block-commitment changes,
and rules on the Ironwood flags.

### 3.7 The turnstile

Crossing Orchard → Ironwood **publicly reveals the amount moved**. Sender and recipient
remain shielded. This is a deliberate supply-verification checkpoint. (Verified,
ZIP 318.)

**Migration state (Reported, context only):** migration is voluntary with no deadline.
Trackers reported roughly 85–90% of Orchard value migrated by late September 2026, with
a few hundred thousand ZEC still in Orchard. Figures come from news sites and
third-party trackers that measure differently (pool balance vs cumulative inflow vs net
migration) — **do not quote a specific number**.

### 3.8 NU7 — what is coming

- Testnet activated at block **4,465,026** on 2026-10-04. (Reported across outlets, and
  **Verified on the Zebra side** — the 7.0.0-rc.0 release notes name the same testnet
  height.)
- **Block time drops 75s → 25s.** (Reported.) This invalidates every height↔time
  assumption: confirmation targets, expiry heights, "N blocks ≈ M minutes" UI copy,
  polling intervals.
- Also reported: a fee-backed sustainability mechanism, and **retirement of v4
  transactions**, which would strand legacy Sprout funds. Confirm against the NU7
  deployment ZIP before relying on it.
- Zebra 7.0.0-rc.0 states mainnet operators need not upgrade yet and **no mainnet NU7
  height is set**. (Verified.)
- Mainnet decision **2026-10-20**; target activation **2026-11-05**. (Reported.)

### 3.9 Canonical repositories

| What | Where | Note |
|---|---|---|
| Specs | https://github.com/zcash/zips | `zips/zip-NNNN.md`; rendered at zips.z.cash |
| Node | https://github.com/ZcashFoundation/zebra | Zcash Foundation |
| Rust libraries | https://github.com/zcash/librustzcash | Electric Coin Company |
| Wallet daemon | https://github.com/zcash/zallet | Beta |
| Indexer | https://github.com/zingolabs/zaino | Zingo Labs |
| Dev stack | https://github.com/ZcashFoundation/z3 | Zebra + Zallet + optional Zaino |
| ~~zcashd~~ | https://github.com/zcash/zcash | **ARCHIVED — do not use or cite** |

Active downstream forks exist and are legitimate, not impersonations:
`zingolabs/librustzcash`, `pacu/librustzcash`, `piatoss3612/wallet-libraries`
(Zakura stack), `logos-co/logos-zcash-wallet-core-module`. They are **not canonical** —
pin upstream unless you have a specific reason.

---

## 4. librustzcash

### 4.1 What it is

The Electric Coin Company's Rust workspace implementing the Zcash protocol and a
complete wallet framework. **It is not a node.** The README states plainly that the only
way to check Zcash consensus validity is to use a Zcash consensus node, and that its
parsing APIs check **only a subset** of validity constraints and are **not necessarily
stable over time**.

Standing disclaimer from the repo: *"These libraries are under development and have not
been fully reviewed."* The repo describes itself as a work in progress.

**Licence:** dual Apache-2.0 / MIT, at your option. Contributions under the same terms.

### 4.2 Full crate map (Verified, README)

The README organises crates under three headings.

**Zcash Protocol**

| Crate | Purpose |
|---|---|
| `zcash_protocol` | Constants, consensus parameters, common types such as value and memo types |
| `zcash_transparent` | Transparent addresses and transaction components |
| `zcash_primitives` | Core transaction type, builders, and proving, signing, serialization |
| `zcash_proofs` | The Sprout circuit and proving system |

**Keys, Addresses & Wallet Support**

| Crate | Purpose |
|---|---|
| `zcash_address` | Parsing and serialization of Zcash addresses |
| `zcash_keys` | Spending keys, viewing keys, addresses, ZIP 32 derivation |
| `zip321` | Parsing and serialization of ZIP 321 payment requests |
| `pczt` | Data types and interfaces for partially constructed transactions |
| **`zcash_client_backend`** | **Wallet framework: storage APIs, chain scanning, fees, proposals** |
| **`zcash_client_sqlite`** | **SQLite implementation of the `zcash_client_backend` storage APIs** |
| `zcash_pool_migration` | Backend-agnostic engine for migrating wallet funds between value pools — **first used for the NU6.3 Orchard → Ironwood migration** |
| `payment_uri` | Parsing and validation of cross-chain payment request URIs |

**Utilities & Common Dependencies**

| Crate | Purpose |
|---|---|
| `f4jumble` | Encoding used for Unified addresses |
| `zcash_encoding` | Bitcoin-derived transaction encoding utilities |
| `equihash` | Proof-of-work protocol implementation |
| `zcash_history` | ZIP 221 Merkle Mountain Range over per-block metadata |

The dependency diagram additionally shows `eip681`, `sapling-crypto`, `orchard`,
`zcash_note_encryption`, `zip32` and `zcash_spec`. These are separately versioned and
the crate list does not describe them.

### 4.3 Versions — pin deliberately

As of 2026-10-14 (Verified: crates.io JSON API + CHANGELOGs):

| Crate | Latest stable | Latest pre-release |
|---|---|---|
| `zcash_client_sqlite` | **0.22.0** (2026-08-19) | 0.23.0-pre.2 (2026-10-14) |
| `zcash_client_backend` | **0.24.0** (2026-08-18) | 0.25.0-pre.2 (2026-10-13) |

`zcash_client_sqlite` crates.io metadata: `max_stable_version` 0.22.0, `max_version` and
`newest_version` both 0.23.0-pre.2. Recent version history: 0.23.0-pre.2 (2026-10-14),
0.23.0-pre.1 (2026-10-06), 0.23.0-pre.0 (2026-10-02), 0.22.0 (2026-08-19), then the
0.22.0-rc series back through 2026-07-26.

**The split that matters:**

- **Stable 0.24.0 / 0.22.0 already has Ironwood.** Ironwood is a supported pool, the
  `pool_migration` module implements ZIP 318, Ironwood notes persist under **pool code 4**.
  This is the line for mainnet as it exists today.
- **The `-pre` cohort is the NU7 line.** Downstream projects targeting NU7 pin
  `zcash_client_backend =0.25.0-pre.x` / `zcash_client_sqlite =0.23.0-pre.x` with exact
  `=` requirements. Three breaking pre-releases shipped in the twelve days to 2026-10-14.

**MSRV is 1.88** (set in 0.24.0 / 0.22.0). The **`orchard` feature is on by default** —
disable default features for a smaller build.

> **librustzcash does not use GitHub Releases.** The releases page says so explicitly
> and its newest entries are from 2023. The per-crate `CHANGELOG.md` files are the
> authoritative release record.

### 4.4 The wallet sync loop (Verified, crate-level docs)

`zcash_client_backend` describes itself as *"A crate for implementing Zcash light
clients."* The design centres on the APIs in `data_api::chain`, with this repeating loop:

1. Determine the required subtree root range; update the subtree roots.
2. Update the chain tip.
3. Get suggested scan ranges; split them into batches.
4. Download blocks into a cache; scan the cached blocks.
5. **On success:** enhance transactions, update wallet balance and sync progress, update
   scan ranges, return to step 2.
6. **On a continuity error:** choose a rewind height, truncate the wallet to that
   height, delete the affected blocks from the cache, then resume from step 2.

**Step 6 is not optional.** Reorg handling is part of the main loop, not an edge case to
bolt on later.

### 4.5 Module layout

**Public modules:** `data_api`, `fees`, `proposal`, `proto`, `scan`, `scanning`, `util`,
`wallet`.
**Private:** `decrypt`.
**Feature-gated:** `sync` (via `sync` or `sync-decryptor`), `serialization` (via
`unstable-serialization`), `tor` (via `tor`, public), `task` (via `sync-decryptor`,
private).
**Re-exports:** `DecryptedOutput`, `TransferType`, `decrypt_transaction` from `decrypt`.
**Deprecated compatibility shims:** `address`, `encoding`, `keys`, `zip321` re-export
their replacements in `zcash_keys` and the `zip321` crate; `PoolType` and `ShieldedPool`
are deprecated aliases for `zcash_protocol` types.

### 4.6 Traits and the send path

Traits you implement or consume: `WalletRead`, `WalletWrite`, `LowLevelWalletWrite`,
`BlockSource`, `InputSource`, `OutputLockStore`.

`zcash_client_sqlite` provides **complete SQLite implementations of `WalletRead`,
`WalletWrite` and `BlockSource`** — use it rather than implementing the traits yourself
unless you have a hard storage requirement.

**Send path:** build a proposal (`propose_transfer`,
`propose_standard_transfer_to_address`, `shield_transparent_funds`) →
`create_proposed_transactions`. Or go through PCZT: `create_pczt_from_proposal` +
`extract_and_store_transaction_from_pczt`.

### 4.7 `zcash_client_backend` CHANGELOG — detail

**0.25.0-pre.2 (2026-10-13)**
- Migrates to `zcash_keys 0.17.0-pre.2`, `zcash_primitives 0.31.0-pre.2` and others.
- **Breaking:** `WalletRead::list_addresses` must now also return the transparent
  receiver at the diversifier index of an exposed unified address that omits one.
- **Behavior:** `propose_standard_transfer_to_address` now pays **every** receiver of a
  unified address, including the transparent one.
- **Fixes:** `InsufficientFunds.available` includes selected transparent inputs;
  `address_receiver_matches_ua` matches the transparent receiver.

**0.25.0-pre.1 (2026-10-06)**
- **Behavior:** unified addresses the library constructs are encoded at **ZIP 316
  Revision 0 when possible, Revision 2 otherwise**. Previously always Revision 2.

**0.25.0-pre.0 (2026-10-02)** — the largest breaking set
- **Signature changes:** `create_proposed_transactions`, `create_pczt_from_proposal`,
  `extract_and_store_transaction_from_pczt` and `shield_transparent_funds` all gain
  `clock: &impl Clock` and `rng: &mut impl CryptoRng`. `http_get_json` gains a
  request-builder closure. `get_latest_zec_to_usd_rate` gains an `rng`.
  `LockOwner::random` requires `rand_core::Rng`. `WalletTx::new` gains
  `transparent_spends`.
- **New required methods:** `WalletWrite::queue_rescan`; behind `transparent-inputs`,
  `WalletRead::get_unspent_transparent_outpoints` and
  `WalletRead::get_transparent_receiver_accounts`, and
  `LowLevelWalletWrite::track_block_transparent_spends`.
- **Renames:** `prune_tracked_nullifiers` → `prune_tracked_spends`;
  `get_txs_spending_transparent_outputs_of` → `get_unknown_fee_spenders_of`;
  `NULLIFIER_MAP_RETENTION_BLOCKS` → `SPEND_MAP_RETENTION_BLOCKS`.
- **Scanning:** `scan_block` takes `SpendIdentifiers` instead of `Nullifiers`; with
  `transparent-inputs` it also takes a `find_account_for_address` closure and returns
  `ScanBlockError`.
- **Address expiry:** payments to expired recipient addresses, or whose expiry would
  exceed the transaction's expiry height, now fail with `Error::RecipientAddressExpiry`.
- **Atomicity:** `put_blocks` must apply a batch **fully or not at all**, and must
  record received transparent outputs.

**0.24.0 (2026-08-18)** — many breaking changes
- **MSRV 1.88**; `orchard` feature enabled by default.
- Most public error enums are now **`#[non_exhaustive]`** — `match` needs a wildcard arm.
- **Ironwood added as a shielded pool** alongside Sapling and Orchard. Once Ironwood is
  active, **proposals paying the Orchard pool are rejected during decoding**.
- **Locking:** output locks move to a new `OutputLockStore` trait; input-source methods
  gain a `lock_filter`; proposal functions gain `lock_inputs` and move from `WalletRead`
  to `WalletWrite`.
- **Balances:** `Balance::total` and `AccountBalance::total` include locked value;
  `unshielded_balance` returns a value rather than a reference.
- **Rewinds:** `rewind_to_height` replaced by `rewind_to_chain_state`, which can fail
  with `RewindBeyondBirthdays`.
- **Transactions:** `store_transactions_to_be_sent` must be idempotent and must unlock
  spent outputs; `create_proposed_transactions` drops `proposed_version`, gains
  `expiry_height`.
- **Renames:** `TransferType::WalletInternal` is now cross-account only;
  `Recipient::InternalAccount` → `InternalShielded`; `SentTransaction::account_id` →
  `funding_account`; `TransparentOutputFilter` → `CoinbaseFilter`.
- **Proposals:** `Step::from_parts` takes an optional anchor height; a shielded step with
  no anchor is rejected.

**0.23.0 (2026-06-02)**
- Dependency migration, and **security**: pulls in fixes for two Orchard advisories,
  **GHSA-ww9q-8r59-xv46** and **GHSA-2x4w-pxqw-58v9**. No API breaks listed, but
  anything pinned below this is vulnerable.

### 4.8 `zcash_client_sqlite` CHANGELOG — detail

**0.22.0 (2026-08-18/19)**

*Breaking / API:*
- MSRV 1.88; `orchard` on by default.
- All public error enums `#[non_exhaustive]`.
- `truncate_to_height` and `rewind_to_chain_state` accept more target heights;
  `RequestedRewindInvalid` still returned when a rewind would destroy needed witness data.
- Truncation now rolls back stored pool migrations **in the same database transaction**.
  Applications calling `MigrationState::truncate_to_height` from a reorg hook must
  **stop doing so**.
- With `spend-index`, transparent spend detection emits `GetSpendingTx` instead of
  `TransactionsInvolvingAddress`.
- `store_transactions_to_be_sent` upserts sent-output records rather than failing on
  repeats.
- Test defaults use isolated in-memory databases.

*Ironwood / ZIP 318:*
- Ironwood value pool persisted behind the `orchard` feature. **Ironwood notes appear
  under pool code 4** in the `v_received_outputs` / `v_tx_outputs` views and in balances.
- New **`pool_migration`** module implementing the migration store traits, with
  **`PoolMigrations`** as the entry point.
- New views **`v_migration_transactions`** and
  **`v_transactions_with_pending_migrations`**. `v_transactions` itself unchanged.
- **`zip318_kind`** column classifies transactions. Rows written before it existed
  default to "no label yet" and need a rescan — **show as unlabeled, never as "not a
  migration transaction"**.
- **`pool_crossing_value`** is non-NULL for wholly shielded internal transactions that
  moved funds into a new pool. **Use it as the classification predicate and as the
  displayed migrated amount**, because `account_balance_delta` is then just the negated
  fee.

*Other additions:* `zewif` feature for importing Zcash Wallet Interchange Format
documents; `transactionally_with_extension` for atomic writes to application-owned
`ext_` tables; configurable anchor retention interval for note commitment tree
checkpoints; watch-only standalone transparent address import (behind
`transparent-key-import`); note locking columns; `UNIQUE` index on
`addresses.cached_transparent_receiver_address`.

*Fixes:* note selection draws oldest eligible notes first; immature coinbase value
reported as pending rather than spendable; `V_0_8_0` now records the true 0.8.0
migration state, with the 0.8.1 state moved to `V_0_8_1`.

*Database migrations:* adds Ironwood tables, columns and views with `note_version`
backfilled to 2 for existing Orchard notes; adds `zip318_kind`, lock columns, and the
watch-only transparent address constraint relaxation; adds the `UNIQUE` index on the
transparent receiver column — it resolves duplicates within an account and across
accounts, and **aborts if a cross-account duplicate cannot be verified**.

*Integrator action:* add wildcard arms to matches on the listed error enums; remove or
update reorg hooks calling `MigrationState::truncate_to_height`; review code depending
on `RequestedRewindInvalid` for uncovered heights; **plan a rescan** so `zip318_kind` is
populated for older transactions; update `spend-index` consumers for the new transaction
data request type.

**0.23.0-pre.0 (2026-10-02)**
- Broad dependency migration: `orchard 0.16`, `sapling-crypto 0.9`, `rand_core 0.10`,
  `zcash_protocol 0.11.0-pre.0`.
- `WalletDb`'s `R` parameter must implement **`rand_core::Rng`** instead of `RngCore`.
- `take_transaction_for_broadcast` (in `orchard_ironwood::PoolMigrations`) takes an extra
  `rng` as its **first** argument.
- `WalletWrite::import_account_ufvk` accepts transparent-only UFVKs; the default address
  is a transparent-only `tu` address.
- `util` types (`Clock`, `SystemClock`, `FixedClock`) now re-export the
  `zcash_client_backend` versions.
- New **`DivergedCheckpoints { pool, height }`** error replaces `CorruptedData` when a
  pool's checkpoints are split around the truncation height.
- Additions: `WalletDb` implements `queue_rescan` and the new transparent methods;
  `WalletSnapshot` and `get_wallet_snapshot`; `put_blocks` records transparent outputs
  and spends.
- Fixes: upgrades from a `support_zcashd_wallet_import` migration run before 2025-09-16
  no longer fail on a NOT NULL constraint; `v_tx_outputs` includes `diversifier_index_be`;
  `v_transactions` no longer multiplies rows by the number of receiving groups; wallets
  with UIVK-imported accounts no longer fail on init or migration; unmined transactions
  with zero expiry no longer fail with a consensus branch ID error when the tip is known.
- *Integrator action:* update `R` bounds `RngCore` → `Rng`; update
  `take_transaction_for_broadcast` calls; handle `DivergedCheckpoints` where
  `CorruptedData` was matched.

**0.23.0-pre.1 (2026-10-06)**
- Adds `SqliteClientError::UnifiedEncoding`.
- Unified addresses and viewing keys written to the database use **ZIP 316 Revision 0
  when representable, else Revision 2**. Previously all Revision 2. **Existing rows are
  not re-encoded.**
- Fixes the `full_account_ids` migration to accept a stored UFVK encoded at a different
  ZIP 316 revision.
- *Integrator action:* parsers of stored encodings must **accept both revisions**.

**0.23.0-pre.2 (2026-10-13/14)**
- `WalletRead::list_addresses` returns the transparent receiver tracked at the index of
  an exposed unified address that omits it, as an `Address::Transparent` entry.
- Unified addresses generated to fill the transparent gap limit are stored with their
  transparent receiver.
- A migration restores transparent receivers to such addresses stored without one by
  0.23.0-pre.1, if the wallet has not exposed them.
- *Integrator action:* expect **additional transparent entries** from `list_addresses`.

### 4.9 Integrator upgrade checklist (from both CHANGELOGs)

1. Move to **MSRV 1.88**; check feature flags, since `orchard` is now default-on.
2. Add **wildcard arms** to every match on the non-exhaustive error enums.
3. Switch RNG bounds to **`rand_core::Rng`**; update `take_transaction_for_broadcast`.
4. **Remove reorg-hook calls** to `MigrationState::truncate_to_height`.
5. **Rescan** so `zip318_kind` is populated; treat the default as *unlabeled*.
6. Accept **both ZIP 316 revisions** when parsing stored encodings.
7. Expect **new transparent entries** from `list_addresses`.
8. Thread `clock` and `rng` through all transaction-creation call sites.
9. Handle `DivergedCheckpoints` and `RewindBeyondBirthdays`.

---

## 5. Zebra

### 5.1 What it is

A Zcash full node written in Rust by the Zcash Foundation. Since `zcashd`'s halt in July
2026 it is **the** consensus node for the network. It validates and serves the chain and
exposes JSON-RPC. It has **no wallet** — `zebra-client` is listed as *planned*, not
built. Wallet functionality lives in Zallet.

### 5.2 Release history and the two version traps

| Version | Date | Status |
|---|---|---|
| **7.0.0-rc.0** | 2026-10-02 | Latest. Activates NU7 on **testnet only** (height 4,465,026) |
| **6.4.2** | 2026-09-25 | Current stable. Fixes a remotely triggerable DoS in V6 transaction processing |
| 6.4.1 | 2026-09-23 | **WITHDRAWN** |
| 6.4.0 | 2026-09-23 | **WITHDRAWN** |
| 6.3.0 | 2026-08-10 | Adds `getdeprecationinfo` |
| 6.2.3 | 2026-07-28 | Optional network hardening; sidecar pinned to `zebra-compat-v1.1.0` follows mainnet past NU6.3 |
| 6.2.2 | 2026-07-24 | Fixes and security; `getblock`/`getblockheader`/`gettxout` stay consistent during reorgs |
| 6.2.1 | 2026-07-22 | Hardens NU6.3-active nodes against CPU exhaustion by rejecting underpaying/malformed shielded transactions early |
| 6.2.0 | 2026-07-17 | Experimental zcashd-compat mode; regtest-only `generatetoaddress` |
| 6.1.0 | 2026-07-17 | Security and features; new `getstandardfee` RPC |

**Trap 1:** the release notes for 6.4.0/6.4.1 say outright **"Do not run Zebra 6.4.0 or
6.4.1."** Use 6.4.2 or later.

**Trap 2: Zebra halts itself at an end-of-support height.** The 6.4.x line's halt height
was moved to approximately **2026-11-02**, just ahead of the tentative NU7 date. A node
installed today on the 6.4 line **stops in under three weeks**. Check
`getdeprecationinfo` (mainnet only, added 6.3.0) and run 7.0.0-rc.0 or newer for
anything running into November.

**Trap 3:** the README's install snippet pins `--tag v6.0.0`, already several security
releases stale. Install the current tag, not the README's.

Releases carry a SHA-256 checksum, a Sigstore build-provenance attestation and a Cosign
signature.

### 5.3 Install and run

- **Docker:** `zfnd/zebra:latest`; expose **8233** (mainnet p2p) or **18233** (testnet);
  mount a volume for chain state. Then `zebrad start`.
- **Pre-built binary:** `cargo binstall zebrad` — Linux x86_64 or aarch64, **glibc ≥ 2.34**.
- **From source:** `cargo install --locked zebrad`, or from a GitHub tag. Needs Rust,
  libclang and a C++ compiler; `protoc` optional. Arch:
  `sudo pacman -S rust clang protobuf`.

### 5.4 System requirements (Verified)

- **CPU:** 4 cores recommended, 2 minimum.
- **RAM:** 16 GB recommended, 4 GB minimum.
- **Disk:** **~300 GB mainnet**, **~10 GB testnet**. Expected to grow. Snapshot sync
  temporarily needs roughly **double** while the archive extracts.
- **Bandwidth:** 100 Mbps recommended; ~300 GB initial download; ~300 GB/month;
  ongoing 10 MB–10 GB/day.
- **Ports:** inbound TCP 8233 (mainnet) / 18233 (testnet) — optional but recommended.
  Outbound any port. DNS access to the Zcash seeders (usually port 53).
- **Latency:** needs some peers with round-trip latency ≤ 2 s.
- The docs give **no expected sync duration**. Zebra has reportedly run on an Orange Pi
  Zero 2W with a 512 GB microSD.

**For a hackathon: do not sync mainnet.** Use regtest (instant) or testnet (~10 GB).

### 5.5 Architecture and crate map (Verified, dev/overview.md)

| Crate | Role |
|---|---|
| `zebra-chain` | Core data structures (blocks, transactions, addresses, keys) and consensus-critical serialization. No internal dependencies |
| `zebra-network` | Zcash P2P protocol; peer connections; wire ↔ internal request/response translation |
| `zebra-script` | Script parsing and context-free validation; initially wraps zcashd C++ code |
| `zebra-consensus` | Block and transaction verification, consensus parameters, checkpoints, mempool verification |
| `zebra-state` | Stores, updates and queries chain state; contextual verification |
| `zebra-rpc` | **The RPC interface — start here for API work** |
| `zebra-client` | Client/wallet functionality — **planned, not built** |
| `zebrad` | The full node binary; loads config, wires components together |
| `zebra-node-services`, `zebra-utils` | Referenced but not described in the overview |

Components are libraries communicating over internal `tower::Service` interfaces —
"microservices in one process". Data flow: blocks from the syncer pass through a block
verifier router to the checkpoint or block verifier, then the transaction verifier, then
state. Mempool transactions go through the transaction verifier into mempool state.
Inbound peer requests and RPC calls read state and mempool, and submit blocks to the
block verifier router.

*Note:* the docs assign contextual verification to `zebra-state` in one section and
context-dependent validation to `zebra-consensus` in another, and do not resolve the
overlap.

### 5.6 RPC surface (Verified, zcashd API migration guide)

**Implemented (Done / Partial / Stub):**
- **Chain and mempool:** `getbestblockhash`, `getblockcount`, `getblockhash`,
  `getrawmempool`, `gettxout`, `z_getsubtreesbyindex`; *partial:* `getblock`,
  `getblockchaininfo`, `getblockheader`, `getdifficulty`, `getmempoolinfo`,
  `z_gettreestate`.
- **Address index:** `getaddressbalance`, `getaddresstxids`, `getaddressutxos`.
- **Transactions:** `getrawtransaction`, `sendrawtransaction` (both partial).
- **Mining:** `getblocksubsidy`, `getblocktemplate`, `getnetworksolps`,
  `getnetworkhashps`; *partial:* `getmininginfo`, `submitblock`; `generate` **regtest
  only**.
- **Network:** *partial:* `getnetworkinfo`, `getpeerinfo`, `getdeprecationinfo`; `ping`
  is a **stub by design**; `addnode` regtest-only and supports only `add`.
- **Validation:** *partial:* `validateaddress`, `z_validateaddress`; **done:**
  `z_listunifiedreceivers`.
- **Zebra-only additions:** `getbestblockheightandhash`, `getstandardfee`,
  `generatetoaddress`, `rpc.discover`.

**Missing relative to zcashd:**
- *Retired (no owner):* `getexperimentalfeatures`, `getmemoryinfo`, `verifychain`,
  `getlocalsolps`, `getgenerate`, `setgenerate`, `getblockdeltas`, `getblockhashes`,
  `gettxoutsetinfo`, `getaddressdeltas`, `getaddressmempool`, the Sprout-only disclosure
  and migration methods, `zcbenchmark`, `setmocktime`, `dumpwallet`.
- *Planned for Zebra:* `getchaintips`, `getspentinfo`, `gettxoutproof`,
  `verifytxoutproof`, `prioritisetransaction`.
- *Deferred to the v2 P2P protocol (#11281):* `clearbanned`, `disconnectnode`,
  `getaddednodeinfo`, `getnettotals`, `listbanned`, `setban`.
- *Undecided:* the REST interface, the `-txexpirynotify` hook.
- *Removed from the sidecar build:* miner RPCs and `addnode`, so misconfigured setups
  fail loudly.

**Behavioural differences that will surprise you:**
- `getinfo` has no `timeoffset`.
- `getblock` accepts **only numeric verbosity**, and accepts `3` (which zcashd rejects);
  omits `chainwork`, `anchor` and related fields.
- `getblockchaininfo` omits `softforks`; some fields are placeholders.
- `getpeerinfo` returns **10 of zcashd's 24** per-peer fields.
- `getrawtransaction` omits `valueSat` and spent-index fields.
- `sendrawtransaction` accepts `allowhighfees` and **ignores it**.
- `submitblock` returns only `null` / `duplicate` / `rejected`, not zcashd's detailed
  reasons.
- `getdifficulty` **errors** on a chain too short to measure, where zcashd returned `1.0`.
- `stop` returns a different message and works **regtest only**.
- **ZMQ is not implemented.** The indexer gRPC service replaces it; `block_notify_command`
  replaces `-blocknotify`.

**RPC changes by version:** 7.0.0-rc.0 — `getblocktemplate` advertises only `time` as
mutable (miners must request a new template rather than altering transactions or the
parent block), `getnetworksolps` returns an error instead of panicking on overflow,
`coinbasetxn.fee` and `depends` adjusted. 6.4.0 — `getblocktemplate` serves precomputed
templates for the current tip. 6.3.0 — `getdeprecationinfo` added. 6.2.0 — regtest-only
`generatetoaddress`. 6.1.0 — `getstandardfee`.

### 5.7 Regtest (Verified, book/src/user/regtest.md)

```toml
[network]
network = "Regtest"

# May be omitted when testing only Canopy
[network.testnet_parameters.activation_heights]
NU5 = 1          # must be >= 1; height 0 is reserved for Genesis

[mining]
miner_address = "t27eWDgjFYJGVXmzrXeVjnb5J3uXDM9xH9v"

# May be omitted if persistent chain state is wanted
[state]
ephemeral = true

# Needed to submit non-coinbase transactions to the mempool
[rpc]
listen_addr = "0.0.0.0:18232"
```

**Generating blocks — two options:**
1. **Internal miner (experimental):** `internal_miner = true` under `[mining]`, build or
   run with `--features "internal-miner"`. Mining starts after ~30 s; confirm via
   "successfully mined a new block" in the logs or a rising tip height.
2. **RPC:** `getblocktemplate` → convert with `proposal_block_from_template()` from
   `zebra-chain` → serialize and hex-encode → `submitblock`. Success returns
   `{ "result": null }`. The `regtest_submit_blocks()` acceptance test is the full
   worked example.

**Activation heights:** by default **all upgrades activate at height 1**. The docs name
`Canopy`, `NU5` and `NU7` as keys. A documented NU7 reissuance example on a fresh chain:
`Canopy = 1`, `NU5 = 2`, `NU7 = 9`, with `initial_nsm_value_balance` and
`nsm_reissuance_height = 12` under `[network.testnet_parameters]`. The reissuance height
must be ≥ 1, not earlier than NU7, and below 2^31. **These are not public testnet or
mainnet heights.**

**Caveats:** regtest is isolated with no peers; **proof of work is not validated**;
custom state with nonzero genesis transparent outputs must be rebuilt, and you must
rebuild custom state whenever consensus parameters change for already-cached blocks.

### 5.8 Zebra known issues and performance (Verified, troubleshooting.md + README)

**Known issues**
- Progress bar estimates can grow extremely large.
- Zebra **gossips and connects to private IP addresses**; an opt-out config is planned
  (#3117), no workaround given.
- Block download and verification **sometimes time out during initial sync** (#5709);
  the full sync still completes reasonably quickly.
- **Experimental Tor support is disabled** until Zebra upgrades to the latest
  `arti-client` (#8328).

**Build and test**
- **GCC 15 breaks the `rocksdb` dependency.** Workaround:
  `export CXXFLAGS="$CXXFLAGS -include cstdint"` before installing.
- Build out of RAM → `export CARGO_BUILD_JOBS=2`.
- Tests time out or OOM → `cargo test -- --test-threads=2`.
- Network-dependent tests → `SKIP_NETWORK_TESTS=1`.
- macOS crash-reporter dialogs on intentional test panics →
  `defaults write com.apple.CrashReporter DialogType none`.

**Performance**
- Sync speed depends mainly on network conditions and peers. Use a release build for
  your native architecture.
- Slow connection → lower `checkpoint_verify_concurrency_limit` (e.g. 1000) and
  `download_concurrency_limit` (e.g. 25) under `[sync]`.
- Too many peers → lower `peerset_initial_target_size` to 25 under `[network]`.
- Excess logging → `filter = 'info'` under `[tracing]`, or debug only specific
  components (e.g. `zebra_network=debug`). Multiple info logs per second is a bug worth
  reporting.
- **Linux TCP throttling:** the default `tcp_slow_start_after_idle=1` resets congestion
  windows between blocks and can cut throughput to **~6 Mbps even on 1–2 Gbps links**.
  Set `net.ipv4.tcp_slow_start_after_idle=0`, `net.ipv4.tcp_congestion_control=cubic`,
  `net.core.default_qdisc=fq_codel`; persist in `/etc/sysctl.d/99-zebra-network.conf`
  and `sudo sysctl --system`. **Apply on the Docker host, not inside the container.**
  Zebra logs a startup warning on Linux when it is enabled.

### 5.9 The 7.0.0-rc.0 state database upgrade — handle carefully

- The state database moves **v28 → v29 automatically, without a resync**.
- **Keep a v28 backup first.** Disabling `state.delete_old_database` does **not**
  preserve it for rollback.
- **Direct database readers — Zallet's and Zaino's Zebra backends — must be upgraded
  together with the node.** v28 readers cannot read v29's wider records. Start the Zebra
  writer before compatible readers.
- Custom testnets need a resync after upgrading.
- Configuration validation is stricter: some previously accepted funding-stream and
  upgrade-height settings are now rejected.
- **Do not rename or symlink versioned directories by hand.** The supported upgrade
  process is the only safe path.

### 5.10 Zebra documentation map

The Zebra book (`book/src/`, rendered at the unreachable zebra.zfnd.org) contains:
`user/requirements.md`, `user/supported-platforms.md`, `user/install.md`, `user/run.md`,
`user/snapshots.md`, `user/docker.md`, `user/tracing.md`, `user/metrics.md`,
`user/health.md`, **`user/zcashd-api-migration-guide.md`**, `user/zallet.md`,
`user/zcashd-compat.md`, `user/zcashd-compat-migration.md`, `user/lightwalletd.md`,
`user/parameters.md`, `user/mining.md` (+ s-nomp and docker variants),
`user/elasticsearch.md`, `user/fork-zebra-testnet.md`, `user/custom-testnets.md`,
**`user/regtest.md`**, **`user/troubleshooting.md`**; and under `dev/`: `overview.md`,
`mempool-specification.md`, diagrams, `state-db-upgrades.md`, `release-process.md`,
`continuous-integration.md`, `zebra-checkpoints.md`, `ecc-updates.md`,
`consensus-refactors.md`, `private-testnet.md`, `crate-owners.md`, and RFCs 0001–0012.

Health endpoints expose `/ready` and `/healthy` for Kubernetes and load balancers.

---

## 6. The surrounding stack

### 6.1 Zallet — the wallet daemon

- **What:** a full-node Zcash wallet in Rust, replacing the `zcashd` embedded wallet.
  Repo: https://github.com/zcash/zallet. User guide at zcash.github.io/zallet.
- **Status: beta.** Breaking changes may happen at any time, **you may need to delete
  and recreate your wallet**, many ported JSON-RPC methods are not yet implemented, and
  the project changes rapidly between betas. The crates are explicitly **not designed to
  be used as a Rust library**.
- **Connection:** after 0.1.0-beta.3, Zallet reads chain data **only from Zebra**, via
  Zebra's **indexer gRPC service** — `zebrad` must be configured with it enabled. Zallet
  **embeds Zaino's libraries directly** rather than calling the standalone Zaino service.
- **Build:** Docker, containerd, and GNU Make; `make` uses StageX for a deterministic,
  reproducible OCI image running as non-root.
- **Migration:** Zallet Alpha.4 added importing `zcashd` `wallet.dat` files
  (`zallet migrate-zcashd-wallet`). (Reported.)

**Wallet RPCs now in Zallet (Done):** `z_getbalanceforaccount`, `z_getnewaccount`,
`z_getaddressforaccount`, `z_listaccounts`, `z_listunspent`, `z_sendmany`,
`z_shieldcoinbase`, `z_viewtransaction`, `z_getnotescount`, `z_getoperationstatus`,
`z_getoperationresult`, `z_listoperationids`, `z_importkey` (Sapling only),
`z_importviewingkey`, `z_exportkey`, `z_exportviewingkey`, `z_converttex`,
`z_gettotalbalance` (deprecated), `listaddresses`, `signmessage`, `verifymessage`,
`decodescript`, `walletlock`, `walletpassphrase`. `getwalletinfo` is partial (balance
fields not yet populated).

**Renames from zcashd:**

| zcashd | Now |
|---|---|
| `getbalance`, `z_getbalance` | `z_getbalanceforaccount` |
| `getnewaddress`, `z_getnewaddress` | `z_getnewaccount` + `z_getaddressforaccount` |
| `gettransaction` | `z_viewtransaction` |
| `listunspent` | `z_listunspent` |
| `sendtoaddress`, `sendmany` | `z_sendfromaccount` or `z_sendmany` |
| `z_listaddresses` | `listaddresses` |
| `encryptwallet` | `walletpassphrase` / `walletlock` (keys are always encrypted) |
| `importwallet`, `z_importwallet` | `z_importkey` per key, or `zallet migrate-zcashd-wallet` |
| `createrawtransaction`, `fundrawtransaction`, `signrawtransaction` | **Zallet's PCZT methods** |
| `listtransactions` | `z_listtransactions` |

**Planned in Zallet:** `addmultisigaddress`, `createmultisig`, `getreceivedbyaddress`,
`getunconfirmedbalance`, `importprivkey`, `listlockunspent`, `lockunspent`,
`listreceivedbyaddress`, `z_listreceivedbyaddress`, `listsinceblock`, `z_mergetoaddress`,
`z_exportwallet`.
**Undecided:** `walletpassphrasechange`, `resendwallettransactions`.
**Retired:** `backupwallet`, `dumpprivkey`, `getrawchangeaddress`, `keypoolrefill`,
`listaddressgroupings`, `settxfee`, Sprout migration methods.

*The Zebra docs note Zallet's status page can lag its changelog, and that the table may
be outdated.*

### 6.2 Zaino — the indexer

- Rust indexer by Zingo Labs, sitting between a validator (Zebra or formerly zcashd) and
  client applications. Serves **both** the lightclient protocol API (the interface
  lightwalletd serves) **and** a JSON-RPC API. Repo: https://github.com/zingolabs/zaino.
- **`zainod` 0.4.1, 2026-06-18** (after 0.3.1 in May and 0.2.0 in March). (Reported, via
  lib.rs.)
- Aims for backward compatibility so lightwalletd users can switch with minimal changes.
- In Z3 it is an optional component: lightwalletd-compatible gRPC on **mainnet 8137**,
  JSON-RPC proxy on **mainnet 8237**.
- **The Zebra book does not mention Zaino at all** — a sign the docs lag the ecosystem.
- A competing indexer, **Zinder**, was described on the forum in August 2026, with its
  own query protocol plus a lightwalletd-compatible runtime and a CI parity suite
  against the reference Go lightwalletd. (Reported.)

### 6.3 lightwalletd with Zebra (Verified, book/src/user/lightwalletd.md)

lightwalletd runs as a separate service in front of Zebra and syncs its own data;
wallets connect to lightwalletd, not to Zebra.

```toml
[rpc]
listen_addr = '127.0.0.1:8232'   # 18232 for testnet
parallel_cpu_threads = 0
enable_cookie_auth = false        # lightwalletd can't do cookie auth; Zebra defaults it on since v2.0.0
```

On the lightwalletd side, `zcash.conf` can be empty when using the default endpoint with
the `zcash/lightwalletd` fork; for a non-default port set `rpcport=28232` and
`rpcbind=127.0.0.1`; for testnet, `testnet=1`.

**The book recommends `zcash/lightwalletd`** because that is what they test against;
other forks have limited support. Wait for lightwalletd to finish syncing before
connecting a wallet.

**Experimental alternative:** Zebra can serve the gRPC interface **directly** via
`lightwalletd_listen_addr`, with no lightwalletd process. Labelled **experimental and
not production-ready**: plaintext, no authentication, should be bound to loopback behind
a TLS reverse proxy, and **untested with light-client SDKs and mobile wallets**.

### 6.4 Z3 — the combined dev stack

https://github.com/ZcashFoundation/z3 — "the grand unification of Zebra, Zaino and
Zallet". A Docker Compose platform running Zebra + Zallet together on **mainnet, testnet
or local regtest**, each network as its own Compose project, with Zaino optional behind
the `indexer` profile.

- **Requires:** Docker Engine with Compose **v2.24.4+**, Git, and `openssl` (regtest
  only). Pulls prebuilt images; build-from-source is an opt-in overlay.
- **Volumes** per network for chain state, Zallet wallet data, the RPC cookie, and Zaino
  data if enabled.
- **Mainnet needs ~300 GB** of disk for chain state.
- Optional **monitoring profile:** Prometheus, Grafana, Jaeger, AlertManager.

| Service | Mainnet | Testnet | Regtest |
|---|---|---|---|
| Zebra p2p | 8233 | 18233 | none (peerless) |
| Zebra RPC (in-network) | `zebra:8232` | `zebra:18232` | `zebra:18232` |
| Zaino gRPC (`indexer` profile) | 8137 | not listed | not listed |
| Zaino JSON-RPC (`indexer` profile) | 8237 | not listed | not listed |
| Grafana (`monitoring` profile) | 3000 | 13000 | 23000 |

Host-side ports for Zebra RPC, health and Zallet RPC are set via `Z3_*` environment
variables; the full per-network matrix is in `z3-contract.yaml`. Zebra's health endpoint
exposes `/ready` and `/healthy`.

The README does **not** claim Z3 is the official or only recommended dev environment,
but it does recommend **regtest for local practice and development because it starts in
seconds with no peers or sync**, and describes running several networks side by side.

---

## 7. Wallet rules after Ironwood

Two wallet-category ZIPs constrain how any Zcash wallet must generate keys, scan the
chain and move funds now that Orchard is sealed. If you use `zcash_client_backend`
≥ 0.24.0 / `zcash_client_sqlite` ≥ 0.22.0, much of this is implemented for you — the
value of knowing it is understanding what the library is doing and where your own code
can break it.

Both are **Draft**, and parts of ZIP 326 are explicitly **provisional**: its
Orchard-spend section is a placeholder until ZIP 2006 defines the same-address
restriction, and its expanded-receiver definition is marked TODO.

### 7.1 `use_qsk` — the new key dimension (Verified, ZIP 326)

Ironwood introduces a quantum spending key path. **Every ZIP 32 account is either
all-`use_qsk = true` or all-`use_qsk = false` — never a mix.**

Hard rules:
- **Do not generate `use_qsk = true` keys in production before NU6.3 mainnet
  activation.** Any account that does gets a birthday after activation. (Mainnet has
  now activated, so this mainly governs how pre-existing accounts are treated.)
- **Do not expose `use_qsk = true` Orchard-protocol receivers or viewing keys before
  activation.**
- **Never send Orchard-pool funds to a `use_qsk = true` account.**
- **After activation, never send Orchard-pool funds to external receivers** — including
  the wallet's own external receivers.
- **Never use knowledge of your own accounts' internal addresses** to send funds from
  one account to another account's internal address.

Deployment: the `use_qsk = true` key restrictions applied **immediately**; the scanning
rules apply **from NU6.3 activation**.

### 7.2 Scanning (Verified, ZIP 326)

- **A single IVK scope covers both pools.** The same incoming viewing key scans
  Orchard-pool and Ironwood-pool note ciphertexts.
- **Ironwood scan range:** from the later of the birthday height and the activation
  height, up to the chain tip.
- **Orchard-pool scan ranges — stop scanning as soon as you may:**
  - Never scan Orchard-pool outputs with `use_qsk = true` keys.
  - For accounts born **after** activation, do not scan Orchard-pool outputs at all.
  - For accounts born **before** activation, do not scan with the external
    `use_qsk = false` IVK after activation.
  - Once the account's Orchard-pool balance is known to be zero at or after activation,
    stop scanning Orchard-pool outputs in descendant blocks.
- **Stop at the first observed `use_qsk`:** once funds are seen in either pool under one
  `use_qsk` value, stop scanning with the other — **even if that transaction is later
  reorged out**.
- **First-funds optimization:** before any Ironwood funds are received, scan only with
  external IVKs, plus internal IVKs for transactions with a **negative
  `valueBalanceIronwood`**. ZIP 326 specifies watermark pseudocode including reorg
  handling — **the watermark rolls back to the fork height, floored at activation, not
  to the tip height.** Implement it from the ZIP, not from intuition.
- **Restoring from seed: treat both `use_qsk` values as possible** and scan with both
  until one is ruled out, **even if your wallet always uses a fixed value**. This is what
  keeps seeds portable across wallet implementations.
- **Outgoing-note detection** uses the OVK (and internal OVK) for the account's single
  `use_qsk` value. No extra dimension is added.
- **Parallel scanning** may run in any order, but only forward scanning from the
  birthday gets the full benefit. Scanning Orchard-pool outputs in parallel can narrow
  the `use_qsk` set to `false` once any such funds are seen.
- **Optional:** a wallet may track whether an account has received any funds, to avoid
  conservatively scanning other users' migration transactions with both scopes.

### 7.3 Receiving (Verified, ZIP 326)

**Orchard-protocol receivers exposed before activation can receive Ironwood-pool funds
afterwards**, and any NU6.3-supporting wallet **must** scan for them. Old addresses do
not stop working. How wallets route payments into Ironwood is out of scope for ZIP 326 —
that is ZIP 318.

### 7.4 Spending in the Orchard pool — provisional (Verified, ZIP 326)

- **Same-address spends:** after activation, each Orchard-pool spend must be paired with
  a fabricated, **zero-valued** output addressed to the spent note's **own receiver**.
- **Randomized ciphertexts:** fill that fabricated output's `enc_ciphertext` with
  **random bytes**, not a real encryption to the receiver. This prevents a quantum
  adversary holding the IVK from linking the nullifier to the address.
- **PCZT handling:** the fabricated output still carries its explicit recipient, value
  and `rseed`. Signers should read a zero value and **treat it as a dummy output**, not
  reject it over a ciphertext mismatch.
- **Provisional** until ZIP 2006 defines the same-address restriction.

### 7.5 The ZIP 318 migration (Verified)

**Purpose:** after NU6.3, ordinary payments within the Orchard pool are disabled, so
wallets need a way to move Orchard funds into Ironwood. A pool-crossing transfer
publicly reveals the amount moved, so the ZIP prescribes a privacy-preserving method.

**Step 1 — Note preparation.** Quantize the migratable balance into **canonical
denominations**: values of the form **n × 10^k ZEC where n ∈ {1, 2, 5}**, from
**0.01 ZEC up to 10,000 ZEC**. Run internal send-to-self transactions creating **one
note per planned part**, each preparation transaction **padded to 16 Orchard actions**.
**Balances below 0.01 ZEC remain unmigrated.**

**Step 2 — Scheduled migration.** Each migration transaction spends **one** Orchard note
and sends **one** denomination to **a single** Ironwood output. Parts are **shuffled**
and broadcast at **randomized intervals — exponential delays with a mean of about 66
blocks** — each using an anchor drawn from **shared boundary heights every 144 blocks**,
never the wallet's own sync point.

**What is public:** the net amount crossing per migration transaction, the block height
(timing), the anchor boundary, the coarse expiry height, and the fixed structure (action
count, fee, no transparent components).
**What stays shielded:** the values and direction of note-preparation transactions, and
whether an Orchard change output exists (hidden by the fixed two-action padding).
**Addresses are never visible on-chain.**

**Wallet obligations — all required, all easy to skip and wrong to skip:**
- **Obtain explicit user confirmation of the full schedule** before any funds leave the
  Orchard pool, since this reveals amounts publicly.
- **Offer Tor and Nym** (or suggest a trusted VPN) and **disclose the IP-correlation
  risk**. Users may proceed without these.
- **Send only to the user's own internal Ironwood address.** **Do not offer Sapling as a
  destination.**
- **Store pre-signed transactions**; treat broadcasting as best-effort background work.
  **Never synchronize and broadcast in the same background window.**
- **On each app launch, reconcile overdue transfers:** send **at most one** overdue
  transfer on open, and warn that doing so may correlate with sync activity.
- **Rebuild** invalidated, expired or fee-changed transactions with **fresh anchors and
  expiries**, and notify the user.
- **Show progress**, and present the on-open fallback as **normal operation, not an
  error**.

The v6 change making anchors authorizing data (ZIP 229, §3.6) is what allows a
pre-signed transaction's anchor to be refreshed without the txid changing — that is why
this schedule is possible at all.

### 7.6 Library support and the silent-failure risk

- `zcash_pool_migration` / the `pool_migration` module (`PoolMigrations` entry point) is
  the supported implementation. **Do not hand-roll this.**
- `zcash_client_sqlite` 0.22.0 classifies migrations via **`zip318_kind`**; rows
  predating the column default to *unlabeled*. **Never render unlabeled as "not a
  migration".**
- Use **`pool_crossing_value`** as the migration predicate and the displayed amount;
  `account_balance_delta` for a shielded pool-crossing is just the negated fee.
- **The timing and anchor rules exist solely for privacy.** A "simpler" migration that
  sends everything at once, or uses the wallet's own sync height as the anchor, is still
  consensus-valid and will still be accepted — **it just deanonymizes the user. Nothing
  fails loudly.**

---

## 8. Gotchas and traps

Ordered by how much time each costs.

### Tier 1 — will sink the project

1. **Your training data and every pre-August-2026 tutorial are wrong about the pool
   model.** Orchard is sealed; Ironwood is the live shielded pool. Code proposing a
   payment to the Orchard pool is **rejected at decode time** by `zcash_client_backend`
   ≥ 0.24.0.
2. **`zcashd` does not exist any more.** Halted 2026-07-18 at block 3,417,100; repo
   archived. Any instruction mentioning `zcashd`, `zcash-cli`, `wallet.dat` or
   `zcash.conf` as a live component is stale. Node = Zebra, wallet = Zallet.
3. **NU7 may activate mid-project.** Mainnet decision 2026-10-20, target 2026-11-05,
   block time 75s → 25s. Pin consensus assumptions, test on testnet (already NU7), and
   do not hardcode "N blocks ≈ M minutes".
4. **Zebra halts itself at an end-of-support height** — the 6.4.x line at approximately
   **2026-11-02**. Check `getdeprecationinfo`; run 7.0.0-rc.0 or newer. **Never run
   6.4.0 or 6.4.1**; 6.4.2 fixes a remotely triggerable DoS.
5. **Do not plan to sync mainnet:** ~300 GB disk and ~300 GB download. Regtest is
   instant; testnet is ~10 GB.

### Tier 2 — will cost hours

6. **Stable and `-pre` crate cohorts do not mix.** Stable (backend 0.24.0 / sqlite
   0.22.0) = Ironwood/mainnet-today; pre-release (0.25.0-pre.x / 0.23.0-pre.x) = NU7,
   breaking weekly. Pin exact `=` versions across the whole cohort.
7. **librustzcash does not use GitHub Releases** — that page stops at 2023. Use the
   per-crate `CHANGELOG.md`. Concluding the project is dormant from the releases page is
   a real trap.
8. **ZIP statuses lag reality.** ZIP 258 still reads *Draft* months after activating;
   ZIP 2005 reads *Proposed*. Do not infer "not live" from the status. ZIP 230, however,
   genuinely **is** Withdrawn — v6 is ZIP 229.
9. **Secondary sources miscite the Ironwood ZIPs constantly** (ZIP 209 / 230 / 318 all
   credited with "the turnstile"). Correct mapping in §3.4.
10. **MSRV 1.88**, and `orchard` is a **default-on** feature in the wallet crates.
11. **Every public error enum is `#[non_exhaustive]`** — every `match` needs a wildcard
    arm. This is a wall of compile errors on first upgrade.
12. **GCC 15 breaks Zebra's `rocksdb` build:**
    `export CXXFLAGS="$CXXFLAGS -include cstdint"`. OOM builds: `CARGO_BUILD_JOBS=2`.
13. **Reorg handling is in the main sync loop**, with an explicit continuity-error
    branch. `rewind_to_height` is gone — it is `rewind_to_chain_state`, and it can fail
    with `RewindBeyondBirthdays`.
14. **Zebra's 7.0.0-rc.0 state DB upgrade (v28 → v29) is one-way**, and
    `state.delete_old_database = false` does **not** keep a rollback copy. Back up
    first; upgrade Zallet/Zaino DB readers together with the node.
15. **ZMQ does not exist in Zebra.** `-blocknotify` → `block_notify_command`; the
    indexer gRPC service replaces ZMQ.
16. **Zebra has no wallet.** If a design assumed `getnewaddress` or `sendtoaddress` on
    the node, redesign: those are Zallet RPCs (renamed — see §6.1), or build on
    `zcash_client_sqlite` yourself.

### Tier 3 — correctness and privacy, fails silently

17. **Seed restore must scan both `use_qsk` values**, even if your wallet generates only
    one. Skipping this loses funds on cross-wallet restore, with no error.
18. **The ZIP 318 schedule is privacy machinery**: denomination quantization, 16-action
    padding, ~66-block mean exponential delays, 144-block shared anchor boundaries,
    never broadcasting in a sync window. A naive all-at-once migration is
    consensus-valid and deanonymizes the user.
19. **Classify pool crossings by `pool_crossing_value`**, not `account_balance_delta`.
    Unlabeled `zip318_kind` means "needs rescan", not "not a migration".
20. **Both ZIP 316 revisions appear in stored encodings** since 0.23.0-pre.1, and
    existing rows were not re-encoded. Parsers must accept Revision 0 and Revision 2.
21. **Recipient address expiry can fail a send** (`Error::RecipientAddressExpiry`).
22. **`put_blocks` must be atomic and `store_transactions_to_be_sent` idempotent** if
    you implement the traits yourself. Partial application corrupts the wallet.
23. **Zebra's built-in `lightwalletd_listen_addr` gRPC is experimental**, plaintext, no
    auth, untested with light-client SDKs and mobile wallets. Loopback-only for a demo;
    never ship it.
24. **Zebra connects to and gossips private IP addresses**, and experimental **Tor
    support is disabled** pending an `arti-client` upgrade. If your pitch involves
    node-layer network privacy, check this first.
25. **Linux TCP defaults throttle block propagation** to ~6 Mbps on fast links. Set the
    sysctls on the Docker **host**.
26. **Two Orchard security advisories** (GHSA-ww9q-8r59-xv46, GHSA-2x4w-pxqw-58v9) were
    fixed via dependency bumps in `zcash_client_backend` 0.23.0 (2026-06-02). Anything
    pinned below that is vulnerable.
27. **A fork reports a release candidate calling an API unpublished in its crates.io
    release.** Mixing stable and `-pre` crates will not resolve. (Reported.)

### Things that look alarming but are fine

- *"These libraries are under development and have not been fully reviewed"* is the
  standing librustzcash disclaimer, not a new warning.
- Everything is below 1.0. Normal for this ecosystem; the CHANGELOGs are disciplined and
  breaking changes are documented.
- Zallet being in beta with missing RPCs is expected and stated upstream.

---

## 9. Build paths and open ground

*The facts above are sourced; the judgments in this section are mine and should be
pressure-tested against the event's actual rules.*

### 9.1 Four viable architectures

**A. Regtest + Z3 — fastest path to a working demo.** Instant blocks, all upgrades at
height 1 by default, full RPC, disposable state. Costs: nothing is real, no public
explorer link, judges may discount it. Best when the demo is *your* logic, not Zcash
liveness.

**B. Testnet — real network, affordable.** ~10 GB of state, and **already on NU7**, so
you build against the next ruleset. Best for a demo that must look real, and for
anything NU7-dependent. Watch: testnet-green ≠ mainnet-valid.

**C. Light client on librustzcash — the "real wallet" path.**
`zcash_client_sqlite` + `zcash_client_backend` syncing via lightwalletd or Zaino gRPC.
Highest technical credibility; most code. You own the sync/reorg loop and the ZIP 326
scanning rules.

**D. RPC-only against a node you don't run.** JSON-RPC to Zebra, or gRPC to a public
lightwalletd/Zaino. Best for explorers, analytics, dashboards. Costs: no wallet RPCs on
Zebra — sending needs Zallet or your own transaction construction.

### 9.2 Recommendation

**Start at A, demo on B.** Build and iterate on regtest where blocks are instant and
state is disposable, then point the same binary at testnet for the recorded demo. That
keeps the 300 GB mainnet sync, the ~2026-11-02 Zebra 6.4.x halt and the 2026-10-20 NU7
mainnet decision entirely off your critical path.

If the project is wallet-shaped, add C and pin the **stable** cohort
(`zcash_client_backend` 0.24.0 / `zcash_client_sqlite` 0.22.0) unless you specifically
need NU7 — the `-pre` cohort shipped three breaking releases in the twelve days to
2026-10-14 and will move again during the build.

### 9.3 Where the interesting ground is right now

Openings created by the last three months, not evergreen ideas — which is what makes
them defensible in a judged setting.

1. **ZIP 318 migration UX.** The spec imposes an unusually demanding interaction model:
   publicly-visible amounts needing explicit informed consent, a multi-week randomized
   broadcast schedule, best-effort background sends that must never coincide with a
   sync, on-launch reconciliation of at most one overdue transfer, rebuilds on expiry.
   Roughly 10–15% of Orchard value is reportedly still unmigrated. Most wallets will
   implement this joylessly.
2. **Post-Ironwood correctness tooling.** The ZIP 326 scanning rules (`use_qsk`
   dimension, both-values-on-restore, watermark rollback floored at activation) are
   subtle and silently wrong when mishandled. A conformance suite or fuzzer against the
   published pseudocode is real, verifiable work.
3. **Supply / turnstile transparency.** The turnstile exists to make supply auditable,
   yet existing trackers disagree because they measure pool balance, cumulative inflow
   and net migration differently. A tracker that states its methodology and reconciles
   all three would be genuinely useful.
4. **NU7 readiness.** 75s → 25s breaks every height↔time assumption in downstream
   tooling. A linter or audit tool for that bug class has a short window of maximum
   relevance.
5. **Documentation that matches reality.** The Zebra book never mentions Zaino; the
   README pins a stale tag; librustzcash has no release notes on GitHub. Small,
   mergeable contributions that prove you read the stack.

---

## 10. What to re-verify, and open questions

### Re-verify if reading this more than ~2 weeks after 2026-10-14

- **NU7 mainnet decision and height** — decision was due 2026-10-20.
- **Crate versions.** The `-pre` cohort moves weekly; the stable line may have shipped
  0.25.0 / 0.23.0 finals.
- **Zebra's current release** and its end-of-support halt height.
- **ZIP statuses**, several of which should move from Draft to Final.

### Known gaps in this research

- **NU6.2 is unverified.** Height and nature come from a release note and a proposed
  wiki edit that do not fully agree.
- **The NU7 feature list is Reported, not Verified** — I did not reach the NU7
  deployment ZIP. The 25-second block time, fee-backed sustainability mechanism and v4
  retirement all need confirming against the ZIP.
- **ZIP 255 (NU6.1) and ZIP 2006 texts were not fetched.** ZIP 2006 matters because it
  will finalize the provisional Orchard-spend rules in ZIP 326.
- **Migration percentages are secondary-source only** and the trackers disagree by
  methodology.
- **No worked end-to-end `zcash_client_sqlite` tutorial exists** that I could find for
  2026. The closest practical references are the crate's own tests, and downstream
  projects such as `logos-co/logos-zcash-wallet-core-module` (pins the NU7 pre-release
  cohort; its README returned 404 on the `main` branch, so it was not read).
- **`zips.z.cash`, `docs.rs` and `zebra.zfnd.org` were unreachable** from this
  environment; all equivalent content was read from GitHub instead.

### Questions that change what to build

1. **The event's rules and judging rubric** — sponsor prizes usually dictate which of
   the four paths scores.
2. **New project, or built on an existing codebase?**
3. **Language constraint?** The stack is Rust-first. TypeScript or Python means
   RPC/gRPC (path D) or wrapping the Rust via FFI/WASM — a real cost to plan for.

---

## 11. Source index

**Primary sources fetched in this session**

*Specs (`zcash/zips`, via raw.githubusercontent.com)*
- `zips/zip-0258.md` — Deployment of the NU6.3 Network Upgrade
- `zips/zip-0229.md` — Version 6 Transaction Format
- `zips/zip-2005.md` — Ironwood Quantum Recoverability
- `zips/zip-0318.md` — Orchard to Ironwood Migration
- `zips/zip-0326.md` — NU6.3 Consequences for Wallets
- Directory listing of `zips/` (confirmed zip-0250 … zip-0317 filenames)

*librustzcash*
- `README.md`
- `zcash_client_backend/src/lib.rs` (crate-level docs)
- `zcash_client_backend/CHANGELOG.md`
- `zcash_client_sqlite/CHANGELOG.md`
- https://github.com/zcash/librustzcash/releases (confirms GitHub Releases is unused)
- https://crates.io/api/v1/crates/zcash_client_sqlite (version metadata)

*Zebra*
- `README.md`
- https://github.com/ZcashFoundation/zebra/releases
- `book/src/SUMMARY.md`
- `book/src/user/zcashd-api-migration-guide.md`
- `book/src/user/regtest.md`
- `book/src/user/requirements.md`
- `book/src/user/lightwalletd.md`
- `book/src/user/troubleshooting.md`
- `book/src/dev/overview.md`

*Other*
- `ZcashFoundation/z3` — `README.md`
- `zcash/zallet` — `README.md`

**Secondary sources** (orientation and cross-checking only; every claim drawn from them
is labelled Reported or Unverified above): z.cash upgrade and network pages, Zcash
Community Forum engineering updates, ZecHub wiki, lib.rs crate pages, and crypto news
coverage of the Ironwood and NU7 activations (The Block, CoinDesk, Cointelegraph,
CryptoBriefing, KuCoin, and the ironwood.live / zecstats.org / cipherscan.app trackers).

---

*End of briefing. Compiled 2026-10-14.*
