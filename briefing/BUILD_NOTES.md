# BUILD_NOTES

Things learned while building that the code and the other briefing files do not say. Decisions go
to `DECISIONS.md`; recorded runs go to `evidence/`. This file is for the facts that cost time.

## Quality pass: adversarial reorg, property tests, CI, 2026-10-20

- **rustfmt and rustc resolve a test target's nested modules differently.** For the target root
  `tests/store.rs`, rustc resolves `mod store { pub mod fixture; }` to `tests/store/fixture.rs`
  (the crate root's own directory, plus the inline module's name). rustfmt instead treats the
  target file's stem as the directory and ignores the inline wrapper, so the two agree only
  because that wrapper happens to be named after the file. Declaring `mod adversarial { ... }`
  inside `tests/verify.rs` compiles, but `cargo fmt` fails with "failed to resolve mod
  `foreign_gate`: tests/verify/foreign_gate.rs does not exist". `#[path = "adversarial"]` on the
  inline module makes both tools resolve `tests/adversarial/`, which is how one attack directory
  is shared between several targets.
- **One real proof per test target, not per file.** Each integration-test target is its own
  process, so a `OnceLock` fixture is built once per target, not once per run. Giving the attacks
  their own target would have rebuilt the verifier's four K=12 proofs (about 170 s) and the
  prover's one (about 55 s) on every gate run; instead each existing target declares the attack
  modules whose fixture it already pays for.
- **`rust-toolchain.toml` pins 1.91, and that pin rebuilds everything once.** The next cargo call
  after adding it makes rustup install 1.91.1 and invalidates every artifact built with the
  previous default toolchain, so the first gate run after the pin rebuilds the whole dependency
  graph in release.
- **Property tests are seeded by case number.** `StdRng::seed_from_u64(seed)` per case, every
  failure collected as `(seed, reason)` and the list asserted empty, so a failure names the case
  and nothing depends on OS entropy. The generators are biased on purpose (70% of windows put h1
  on the nullifier step, 80% put h0 below h1): uniformly random triples almost never make a valid
  window, and a property that only ever exercises refusals proves little. A28 and A32 assert that
  every outcome was actually reached.
- **`git grep`, not `git ls-files | xargs grep`, in the leak scan.** A pattern beginning with `-`
  is read as an option by grep, and with `|| true` on the end the check passed silently while
  scanning nothing. `git grep -e <pattern>` separates "no match" (exit 1) from "could not run"
  (exit above 1), which the scanner now reports as a failure, and it skips tracked files that have
  been deleted from the working tree. Both patterns were then confirmed against a planted canary.

## M2 step 2 (challenge and marker store), 2026-10-17

- **BLAKE2b personalisation is 16 bytes.** `unstirred/chal/v1` is 17 and `blake2b_simd` panics on
  a longer personalisation. The hash uses `UnstirredChal_v1`. The raw challenge is what the
  signature binds; the file stores only this hash.
- **A consumed challenge is allowed through the read-only check.** Markers are inserted before
  the consume, inside `BEGIN IMMEDIATE`. Replaying an admitted package therefore hits
  `UNIQUE(context, marker)` and comes back `ALREADY_USED_IN_CONTEXT`, and the transaction rolls
  back. Rejecting "consumed" before the proof would hide that replay behind
  `CHALLENGE_UNKNOWN_OR_EXPIRED`. A consumed challenge whose five markers are all new fails the
  `UPDATE` (zero rows) and is `CHALLENGE_UNKNOWN_OR_EXPIRED`, also rolled back. Unknown, expired
  and wrong-context challenges are refused before the proof. `expires_at > now`, so the instant
  of expiry is already expired. TTL is 600 seconds.
- **rusqlite 0.37 turns on extended result codes in `open`.** A unique violation is
  `SQLITE_CONSTRAINT_UNIQUE` (2067), not primary code 19. The store also treats
  `ConstraintViolation` whose message contains `UNIQUE` as the marker conflict, so a build that
  did not enable extended codes still maps to `ALREADY_USED_IN_CONTEXT`. A `NOT NULL` failure
  does not contain that word and stays a store error.
- **`busy_timeout` is per connection.** WAL persists in the file; the timeout does not. A second
  connection sees `journal_mode=wal` and rusqlite's own default (5000ms) until something sets it.
  The store sets 10s on the connection it opened. Read `PRAGMA busy_timeout` on that connection.
- **Dropping an uncommitted `Transaction` rolls it back.** A05d returns before `commit` and does
  not call `rollback` itself. The interrupted challenge stays unconsumed and its markers are absent.
- **The five markers of one proof are not a set you can compare across proofs blindly.** The real
  note's marker is stable for the same note and the same context, which is why a second proof of
  the same coins collides. The padding slots use fresh nullifiers, so the other four markers
  differ every proof. A different context changes the round id, so all five differ. A05's test
  asserts the two same-context proofs actually share a marker.
- **An integration test of the verifier cannot name `zcash_client_sqlite`.** It is not a direct
  dependency. The account type has to be inferred, which is why the three-proof fixture proves
  through a closure instead of a helper with a written signature.
- **The WAL sidecar is `<path>-wal`.** `PRAGMA wal_checkpoint(FULL)` folds it into the main file.
  A dump test has to read both, and must not search the bytes for a short note position: those
  integers show up inside unrelated encodings. Compare the integer columns with the position and
  the value, and search the bytes for the raw challenge, its hex, a long slice of the proof and
  the note value when that decimal has at least six digits. The marker blobs are supposed to be
  in the file.
- **On 2026-10-17 the owner's Zebra was still syncing, and the tip moved during the session.**
  `getblockcount` was 193,200 in the morning note, then 1,675,600, 1,791,200, and 1,836,400 at
  the start of the live admit (`evidence/2026-10-17-admit-two-rooms.txt`). All of those are
  short of h0 4,505,790, so that admit used one lightwalletd source.

## M2 step 1 (two-source root authority), 2026-10-17

- **The Zebra `z_gettreestate` shape was taken from source, not memory:** zebra-rpc 7.0.0-rc.0,
  `zebra-rpc/src/methods/trees.rs`, type `GetTreestateResponse` — `hash`/`height`/`time`, optional
  `sprout`, then `sapling`, `orchard` and optional `ironwood` (only present from NU6.3, omitted
  otherwise), each a `Treestate { commitments: Commitments }` whose `Commitments` serializes
  `finalRoot` and `finalState` as hex strings. The network name is the BIP70 `chain` field of
  `GetBlockchainInfoResponse` in `zebra-rpc/src/methods.rs`: z_gettreestate carries no network
  field, so the network check is a separate `getblockchaininfo` call.
- **The root is parsed from `finalState`, not `finalRoot`.** Both fields come from the same
  server, so they cannot cross-check each other; going through `finalState` (the zcashd-compatible
  `CommitmentTree` serialization) lets the Zebra source reuse the lightwalletd parsing path, so
  height, network and empty-tree rules stay one code path. Byte order also differs: Zebra's
  `finalRoot` is `bytes_in_display_order`, not the frontier `to_bytes()` order the lightwalletd
  path produces.
- **The owner's Zebra node (7.0.0-rc.0, `http://127.0.0.1:18232`, no cookie auth) answered
  `getblockcount` → 193200 on 2026-10-17** — short of h0 4,505,790, and Ironwood only exists from
  testnet height 4,134,000, so the live node cannot serve a fixture for h0 today. The Zebra-shape
  fixtures are derived from Zebra's own response types, plus one real snapshot body (Zebra's
  `z_get_treestate_by_hash@custom_testnet.snap`, height 0, no `ironwood` field).
- **An unusable source makes the whole root check INCONCLUSIVE, and "unusable" includes an honest
  tip below h0.** Unreachable, malformed, wrong height, or tip < h0 all yield
  `ROOT_SOURCE_UNAVAILABLE`; only answers for h0 with tip ≥ h0 are usable, those must agree on the
  root exactly, and the governing tip is the lower of the answers. This is why an unsynced Zebra
  produces INCONCLUSIVE rather than a misleading `REFUSED / WINDOW_INVALID`.
- **An empty `ironwood_tree` field is now refused.** It used to parse silently as the empty tree's
  root; both sources now report `ROOT_SOURCE_UNAVAILABLE` for it.
- **`CommitmentTree::from_frontier` needs incrementalmerkletree's `legacy-api` feature.** The
  verifier declares it as an explicit dev-dependency feature rather than relying on feature
  unification from another crate.
- **`&Vec<T>` → `&[T]` unsize coercion does not drive `.collect()` inference.** Passing `&vec`
  where `&[SourceOutcome]` was expected left `collect()` without an element type; annotate the
  `Vec` (`let per_source: Vec<SourceOutcome> = …`).
- **The single-source note and `verdict_object` live in the verifier library**, so the A27 test
  asserts the exact text the CLI prints ("ROOT FROM ONE SOURCE: not independently checked") in
  both the human output and the JSON verdict object.
- **`CommitmentTree` and `Frontier` are not re-exported by `zcash_primitives`**; import them from
  `incrementalmerkletree::frontier`. `Frontier::try_from(NonEmptyFrontier)` returns `FrontierError`,
  which implements neither `StdError` nor `Display`, so `map_err` has to format it with `{e:?}`.
- **The empty-`ironwood_tree` refusal fires in the conversion into the lightwalletd `TreeState`**
  (`proto_tree_state`), before `ironwood_root_at` runs — tests for it must assert on the
  conversion's outcome, not the later root check.
- **Zebra's JSON-RPC is plain HTTP POST via hyper-util's legacy client, not gRPC** (the lwd source
  is tonic); `ZebraSource` accepts `http://` only.
- **Clippy `useless_conversion` cost one gate run:** `.map_err(String::from)` on a
  `Result<_, String>` is a redundant conversion; use the error as-is.

## M1 step 5 (`unstirred verify`), 2026-10-17

- **A shared synthetic harness lives in the prover lib behind a `test-harness` feature.**
  `crates/unstirred-prover/src/testing/` is the old `tests/harness/` moved into the library so the
  verifier's tests build the same synthetic wallet instead of a copy. The feature is off by
  default, so `cargo test -p unstirred-prover` alone no longer compiles the integration tests:
  they need `--features test-harness` (the zcash crates' `test-dependencies` pattern), and
  `scripts/gate.sh` now runs clippy and tests with `--all-features`. The verifier enables it as a
  dev-dependency, which is ordinary feature unification and needs no workspace flags.
- **`redpallas::Signature` converts to bytes by reference only.** `[u8; 64]::from(signature)` does
  not exist; `[u8; 64]::from(&signature)` does (orchard 0.15.5).
- **`SpendAuthorizingKey::randomize` takes the randomizer by reference** (`&pallas::Scalar`).
- **A one-byte proof flip must reach step 8 to prove the ordering.** The binding message covers
  the instance, session and challenge but not the proof bytes, so a tampered proof passes
  signature verification and is refused at the circuit with `PROOF_INVALID`; a tampered *instance
  root* dies earlier at step 5 with `ROOT_MISMATCH`. Tests that flip one hex digit (keeping the
  field parseable) assert the code, not just the verdict kind, so ordering regressions show up.
- **The CLI parses the manifest before the root fetch, not the engine's `verify_package`.**
  Step 2 needs the tip and the fetch needs h0 and the network, so the command line runs core's
  `parse_manifest` first (a malformed manifest is refused before any network) and then calls
  `verify_parsed`; `verify_package(json, …)` exists for callers that have only text. A root
  source that fails is `INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE` and is reported before any
  verdict, which is the task's "any root-source failure" rule.
- **`unstirred verify` cannot test the real chain in unit tests.** The one network-touching
  test (`A26`) points at `127.0.0.1:1`, where nothing listens; every other check is pure over
  the injected roots and tip.
- **`PublicInstance::decode` can surface `BAD_SIGNATURE` on its own.** The rk in the instance is
  rebuilt as a curve point, so a package whose rk is not a valid point is refused at step 3's
  decode, before the signature check; the engine's ordering inherits this core behavior.
- **After a session interruption the shell's cwd can reset to the home directory.** A background
  `./scripts/gate.sh` started from there dies instantly (no such file) and the output never
  appears; check pwd or use absolute paths before long runs.

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
