# BUILD_NOTES

Things learned while building that the code and the other briefing files do not say. Decisions go
to `DECISIONS.md`; recorded runs go to `evidence/`. This file is for the facts that cost time.

## M1 step 4 (prove, sign, package), 2026-10-16

- **The delegation builder queries the IMT provider for its own padding nullifiers.**
  `build_delegation_bundle` fills the unused slots of the five with synthetic zero-value notes and
  calls `imt_provider.non_membership_proof(padding.real_nf)` for each one, against the real root.
  A pre-fetched map of the selected notes' proofs is therefore not enough: the provider has to be
  live during the build, which is why `PirNullifierOracle` holds the client and caches answers.
  One note selected means four extra PIR queries per proof.
- **`pir_client::fetch_proof` returns `anyhow::Error`.** There is no typed variant, so "this
  nullifier is in the set" and "the server stopped answering" cannot be told apart from the error.
  The prover reports that case as a local error that claims no verdict, and keeps
  `PIR_UNAVAILABLE` for the failures it can identify (the `/root` handshake and the connection).
- **The `zcash_client_backend` test harness derives its account from 32 zero bytes**
  (`Secret::new(vec![0u8; 32])` in `data_api/testing.rs`), which is not a BIP 39 seed. A harness
  wallet can never be driven through the age-encrypted-mnemonic path, so the identity layer is
  tested against the published all-zero-entropy BIP 39 vector instead, and the synthetic proof
  test takes the harness's own spending key through `spend_authority_for`.
- **`orchard 0.15.5` key types do not zeroize on drop.** `SpendingKey`, `SpendAuthorizingKey` and
  `FullViewingKey` implement no `Drop` and no `Zeroize`, so "zeroize after use" can only mean the
  buffers this crate owns: the decrypted mnemonic and the seed are `secrecy::SecretVec` and are
  zeroized, and the spending key is derived inside one function, never copied to bytes, never
  printed and never written. A heap copy of the key may outlive the proof; that is the library's
  behaviour, not something this crate can fix without forking it.
- **`Instance::from_parts` rebuilds `rk` from `rk_x` plus the parity of `rk_y`.** A package that
  carried a wrong `rk_y` with the right parity is silently normalised to the real one, so the
  wire format effectively carries x and a sign bit. That is not a soundness hole (the proof pins
  both coordinates), but it means a round-trip test cannot detect a tampered `rk_y` of the same
  parity.
- **`bip0039::Mnemonic`'s `Debug` prints the phrase.** The struct itself holds `Zeroizing<String>`
  and `Zeroizing<Vec<u8>>`, so it does wipe itself on drop, but `#[derive(Debug)]` on anything that
  holds one would print the holder's mnemonic. Keep it a local, never a field.
- **`age 0.12.1` brings its own `secrecy 0.10.3`, which does not unify with the zcash cohort's
  `secrecy 0.8.0`.** Both are in the lock. Our own secret buffers use `secrecy` (0.8, the one the
  zcash crates use), and anything handed back by `age` has to be read through `age::secrecy`:
  `age::x25519::Identity::to_string()` returns a 0.10 `SecretString`, so `ExposeSecret` has to
  come from `age::secrecy` or it will not resolve. `age` also drags in `i18n-embed` and a second
  `toml` (0.5.11, through `find-crate`) as proc-macro-time dependencies.

## M1 step 3 (private nullifier lookup), 2026-10-16

- **A spent nullifier shows up two different ways, depending on parity.** In the K=2 punctured-range
  tree each nullifier is either an outer boundary of a leaf or that leaf's `nf_mid`. If it is a
  boundary, `pir_client::fetch_proof` *errors* ("nullifier not found in Tier 1 leaf scan"), because
  no leaf interval contains it. If it is `nf_mid`, a proof comes back and fails `verify`. So the
  lookup has to treat both a per-note fetch failure and `proven_unspent == false` as "spent"; only
  one of them is reachable for any given note, and which one is not predictable.
- **`voting-circuits 0.12.2` cannot verify an IMT proof out of circuit.** `protocol_hash` is a
  private module, `ImtProofData` has no `verify`, and `build_nullifier_list` is behind
  `unstable-internal-api`. The only public Poseidon is `gov_null_hash` (a `poseidon_hash_3` with a
  misleading name) and `derive_nullifier_domain` (fixed first argument). Out-of-circuit checking
  therefore goes through `imt-tree`'s `ImtProofData::verify`, which `pir-client` re-exports. Both
  crates hash with `halo2_gadgets` `P128Pow5T3`, so the two agree.
- **`pir-export` is not on crates.io and never will be from a release.** `cargo publish` strips
  path dev-dependencies, and it strips their feature references too: published `pir-client 0.7.4`
  has no `pir-export` dev-dep and its `upstream` feature lost the `pir-export/upstream` entry. So
  the fixture helpers `build_ranges_with_sentinels` / `build_pir_tree` / `export_for_layout` that
  `pir-client`'s own tests use are unreachable from a registry pin. That is why A04 builds its
  nullifier set with `voting_circuits::delegation::SpacedLeafImtProvider` instead of real tier
  blobs. Real tier blobs in a test would need a git or path dependency on the checkout.
- **`orchard::note::Nullifier::inner()` is only public under `unstable-voting-circuits`**, which
  `voting-crypto-deps/lrz-orchard` happens to enable for the whole graph. Relying on that is
  relying on feature unification from another crate; `to_bytes()` plus `Fp::from_repr` gets the
  same field element without it.
- **`pir-client` does not check the network or the height.** `with_transport` validates the pool,
  the dataset version, the tier layout and the YPIR shape, and nothing else. A server synced to
  another height, or on mainnet, would answer happily about the wrong `h1`.
- **The compiled layout is depth 19 = 12 public + 7 private, `poly_len` 4096**
  (`COMPILED_PIR_LAYOUT`), and the client refuses a server advertising anything else. Treat it as a
  protocol version, not a tuning knob.
- **Next step for a live lookup:** build `nf-server` from `v0.12.3` (`ac15ed2`), and re-export
  `pir-data` with `--features upstream`. The snapshot now in `../vote-nullifier-pir/pir-data/test`
  (height 4,506,510) was exported at HEAD with the default `zakura` features; whether its roots
  match the upstream cohort is untested.
- **Two concurrent `cargo test` runs stall `gate.sh` silently.** The second one waits on the build
  lock, and if its output is piped through `tail` nothing is printed at all, so the gate looks hung
  at `== clippy`. Kill stray cargo processes before running the gate.
