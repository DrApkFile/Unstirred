# HANDOFF

State of Unstirred at commit `118564f` plus an uncommitted M1 step 5, written 2026-10-17. For the
next session (agent or human).
Read `briefing/CLAUDE.md` and `briefing/BUILD_CONTRACT.md` first; this file only says where the
work stopped, what is next, and what the machine around the repo is holding.

## 1. Where the build is

Milestone M1 ("first real proof", target 2026-10-17) is **closed**: all five steps are built, and
`unstirred verify` admitted the first real package in a fresh process against the real testnet
roots — recorded in `evidence/2026-10-17-verify-first-proof.txt` (A01's real run).

| Step | What it added | Real-testnet evidence |
|---|---|---|
| M1 step 1 (`ce7c213`) | read-only wallet, held-note set, selection, `unstirred notes` | `evidence/2026-10-16-notes-real-testnet.txt` |
| M1 step 2 (`0573f04`, `cf3e189`) | h0 tree state over gRPC, historical witnesses, `unstirred witness` | `evidence/2026-10-16-witness-real-testnet.txt` |
| M1 step 3 (`825c3e5`, `c15e092`) | PIR non-membership at h1, `unstirred lookup` | `evidence/2026-10-16-lookup-real-testnet.txt` |
| M1 step 4 (`4c975c4`, `118564f`) | proof package schema, the identity and spending-key path, bundle, prove, sign, `unstirred prove`; first real proof | `evidence/2026-10-17-first-proof.json` |
| M1 step 5 (**uncommitted**) | `unstirred-verifier`: pure engine over injected roots (PRD §6 steps 1-8), one-server root read, `unstirred verify` with `--json` | `evidence/2026-10-17-verify-first-proof.txt` (synthetic tests only in CI; the real admission is the recorded run) |

Step 5 is in the working tree, not committed (built under an explicit "do not commit").
`git status --short` lists it; the new files are
`crates/unstirred-verifier/src/{engine,roots}.rs`, `crates/unstirred-verifier/tests/verify.rs`,
`crates/unstirred-verifier/tests/verify/synthetic.rs`, `crates/unstirred-cli/src/verify.rs`, and
the synthetic harness moved from `crates/unstirred-prover/tests/harness/` to
`crates/unstirred-prover/src/testing/` behind a `test-harness` cargo feature.

What step 5 does, precisely: `unstirred verify` runs PRD §6 steps 1-8 in order (manifest, window
against the tip the command fetches, `round_id` vs the instance, instance roots vs the pinned
ones, challenge equality, binding signature under rk from the instance, the Halo2 proof), with
roots from GetTreeState(h0) and GetLatestBlock at one lightwalletd plus an operator-pinned
`--h1-root`. Every root-source failure is INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE. Step 9
(markers), the challenge store and the two-server root authority are M2. New acceptance IDs A25
(context mismatch) and A26 (root source unavailable) are registered in PRD §10 with a
`DECISIONS.md` row; the rest reuse A01, A06, A07, A09, A11.

`unstirred-gate` is still a one-line doc stub.

Last gate run: `evidence/2026-10-17-gate-m1-step5.md` — `GATE: PARTIAL`, exit 0; fmt and clippy
clean, all tests pass (covering A03, A04, A06-A15b, A16-A26, with A01 closed by the recorded real
admission in `evidence/2026-10-17-verify-first-proof.txt`). Open: A02's repo/log scanner as a
gate step, A05/A08/A10/A12 (markers, two-server roots), and every live-service path in
`unstirred-gate`.

## 2. What to do next, precisely

M2. In order:

1. The verifier's root authority: two agreeing lightwalletd servers for the h0 root (the second
   server is not chosen), with `ROOT_SOURCES_DISAGREE` when they differ, and the nf-server
   `verify-root` check for the h1 root instead of the operator-pinned `--h1-root`.
2. The challenge store (single use, expiry, hashed) so the gate issues challenges, and marker
   insertion in one transaction with `UNIQUE(context, marker)` (PRD §6 steps 6 and 9, codes
   `CHALLENGE_UNKNOWN_OR_EXPIRED`, `ALREADY_USED_IN_CONTEXT`) — this is what closes A05 and A08.
3. `unstirred-gate` (`/manifest`, `/challenge`, `/admit`, `gate markers`) and the Holders' Room
   page. The judge path decision (hosted Holders' Room) is due by M3, 2026-10-21.

The standing rule holds: register any new acceptance IDs in `briefing/PRD.md` §10 **before**
writing their tests, with a `DECISIONS.md` row.

## 3. The machine around the repo (not in git)

Paths are absolute because none of this is inside the repo.

- **Holder's wallet (read-only, real testnet):**
  `/home/kingghidorah/Documents/projects/bounties/zecathon/wallet/data.sqlite`
  account `56099e58-aab0-4f60-9c38-b84ae7211c2d`, scanned to 4,506,611.
  Two held Ironwood notes at h0; the selection takes position 552213 (1 ZEC).
  **Never write to it.** Nothing in this repo may create a transaction from it.
- **Gate manifest used for every run so far:**
  `/home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json`
  — `test` / `holders-room` / h0 4,505,790 / h1 4,506,510 / 12,500,000 zatoshis.
- **lightwalletd:** `https://testnet.zec.rocks:443`. PRD §4 wants a second independent server for
  the verifier's root authority; it has not been chosen yet.
- **nf-server:** built from `vote-nullifier-pir` tag `v0.12.3` (`ac15ed2`) — *not* HEAD, see
  `DECISIONS.md`. Needed only for `lookup` and `prove`, not for `verify`. It was running during
  the step 3 and step 4 runs as:
  ```
  cd /home/kingghidorah/Documents/projects/bounties/zecathon/vote-nullifier-pir/nf-server
  ../target/release/nf-server serve --zcash-network test \
      --pir-data-dir ../pir-data/test-v0123 --port 3000 --pir-poly-len 4096
  ```
  Dataset `pir-data/test-v0123`: ironwood, dataset version 2, height 4,506,510, 276,626 ranges,
  depth 19 (12+7), poly_len 4096. Its `circuit_root` is the expected h1 root
  `8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26`.
- **The holder's age identity:** `/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age`,
  an age identity file (one `AGE-SECRET-KEY-…` line). It is what `--identity` takes, and it is the
  key that decrypts the `mnemonic` field of `wallet/keys.toml`. **Its secret-key line was printed
  into the 2026-10-17 session's transcript** while identifying the file's format, so treat that
  identity as exposed and rotate it (re-encrypt `keys.toml` to a fresh identity) before this
  wallet holds anything that matters.

## 4. Reproducing the recorded runs

From a release build (`cargo build --release`), with `M=/home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json`
and `W=/home/kingghidorah/Documents/projects/bounties/zecathon/wallet`:

```
./target/release/unstirred notes   --wallet "$W" --manifest "$M"
./target/release/unstirred witness --wallet "$W" --manifest "$M" \
    --lightwalletd https://testnet.zec.rocks:443
./target/release/unstirred lookup  --wallet "$W" --manifest "$M" --pir http://localhost:3000 \
    --expected-h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
```

```
./target/release/unstirred prove   --wallet "$W" --manifest "$M" \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://localhost:3000 \
    --identity /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age \
    --challenge $(openssl rand -hex 32) --out ./package.json
```

Recorded 2026-10-17 as `evidence/2026-10-17-first-proof.json`. `lookup` and `prove` need the
nf-server above running. `prove` refuses to overwrite an existing `--out` file.

**The run that closed M1** — accepting that proof in a fresh process — is recorded in
`evidence/2026-10-17-verify-first-proof.txt`. To reproduce:

```
./target/release/unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest /home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json \
    --lightwalletd https://testnet.zec.rocks:443 \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

Recorded result: `ADMITTED`, exit 0, with the h0 root from GetTreeState(4,505,790) equal to
`a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` and the chain tip well past
h1 + finality.

`unstirred codes` lists the closed code set.

## 5. Traps that already cost time

`briefing/BUILD_NOTES.md` is the full list and should be read before touching the PIR, witness,
proving or verifying paths. The ones most likely to bite the next step:

- The synthetic harness moved into the prover library behind the `test-harness` cargo feature.
  Plain `cargo test -p unstirred-prover` no longer compiles the integration tests; use
  `--features test-harness`. `scripts/gate.sh` runs clippy and tests with `--all-features`.
- `voting-circuits 0.12.2` cannot verify an IMT proof out of circuit (`protocol_hash` private,
  no `ImtProofData::verify`); out-of-circuit checking goes through `imt-tree` via `pir-client`.
- The PIR stack must stay on the `v0.12.3` cohort (pasta_curves 0.5 / orchard 0.15 / halo2 0.3).
  HEAD moved to Zakura Common 3.0.0 and its field types cannot meet `voting-circuits 0.12.2`.
- A spent nullifier surfaces two different ways depending on parity: either `fetch_proof` errors
  or the proof verifies false. Both mean spent.
- `orchard::note::Nullifier::inner()` is only public under a feature another crate happens to
  enable; use `to_bytes()` + `Fp::from_repr`.
- Two concurrent `cargo test` runs make `gate.sh` look hung at `== clippy`. Kill stray cargo first.
- The delegation builder asks the IMT provider for proofs for its own *padding* nullifiers, so the
  PIR oracle has to stay live during `build_delegation_bundle` (one note selected = four extra
  private queries per proof).
- `age 0.12.1` brings `secrecy 0.10.3` alongside the zcash cohort's `secrecy 0.8.0`: read age's own
  secrets through `age::secrecy`, ours through `secrecy`.
- The `zcash_client_backend` test harness account comes from 32 zero bytes, not a mnemonic, so a
  synthetic wallet cannot be driven through the age path; `tests/prove.rs` uses
  `spend_authority_for` with the harness's own key, and `tests/identity.rs` covers the age path
  against the published BIP 39 vector.
- One real K=12 proof costs about 70 s in `tests/prove.rs` and about 95 s across the verifier's
  seven tests (one shared proof). All tests in a file share one proof on purpose.
- `redpallas::Signature` converts to `[u8; 64]` by reference only; `SpendAuthorizingKey::randomize`
  takes `&pallas::Scalar`.

## 6. Open items

- **Step 5 is uncommitted.** Committing it is the first thing to decide. The gate passes from the
  working tree; `evidence/2026-10-17-gate-m1-step5.md` records that run and the tree it ran on.
- `scripts/gate.sh` still prints `GATE: PARTIAL`: the leak scanner (A02) and the claims check
  (every README claim links to `evidence/`) are not gate steps yet. A02's package-scan substance
  exists as tests (prover and verifier sides); the gate does not scan logs or the repo.
- `unstirred-gate`: not built. The M1 session key is a fresh 32 random bytes with no private half
  (see `DECISIONS.md`); M3 replaces it with a session the gate issues, and issues the challenge
  instead of the holder.
- `DECISIONS.md` has one `open` row: whether to reserve the crates.io names now with a 0.0.1
  placeholder. Owner's call. The planned publish is a single 0.1.0 around 2026-10-26.
- `Cargo.toml` `repository` is still `https://github.com/<owner>/unstirred`.
- Second lightwalletd server for the verifier's two-source root rule: not chosen.
- Judge path 1 (hosted Holders' Room) has to be decided by M3, 2026-10-21.
- Stated limit worth re-reading before any claim about key hygiene: `orchard 0.15.5` does not
  zeroize its key types on drop (`briefing/BUILD_NOTES.md`, `LEAKS.md` L15b).

## 7. Rules that are easy to break under time pressure

- Never show a placeholder as real; show `—`. No number in README, demo or submission without a
  file in `evidence/` naming network, h0, h1, commit, command and date.
- Unknown is never success. Root trouble is INCONCLUSIVE, never ADMITTED.
- A frozen decision in `briefing/BUILD_CONTRACT.md` changes only after a `DECISIONS.md` row.
- Run `./scripts/gate.sh` before claiming anything works, and record the run.
- Deadline: last commit before **2026-10-27 08:00 UTC**, nothing after — not even docs.
