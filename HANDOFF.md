# HANDOFF

State of Unstirred at commit `c15e092` plus an uncommitted M1 step 4, written 2026-10-17. For the
next session (agent or human).
Read `briefing/CLAUDE.md` and `briefing/BUILD_CONTRACT.md` first; this file only says where the
work stopped, what is next, and what the machine around the repo is holding.

## 1. Where the build is

Milestone M1 ("first real proof", target 2026-10-17) has all four steps built. It is **not closed**:
its definition also needs `unstirred verify` to accept a package in a fresh process, and no proof
has been made from the real testnet wallet.

| Step | What it added | Real-testnet evidence |
|---|---|---|
| M1 step 1 (`ce7c213`) | read-only wallet, held-note set, selection, `unstirred notes` | `evidence/2026-10-16-notes-real-testnet.txt` |
| M1 step 2 (`0573f04`, `cf3e189`) | h0 tree state over gRPC, historical witnesses, `unstirred witness` | `evidence/2026-10-16-witness-real-testnet.txt` |
| M1 step 3 (`825c3e5`, `c15e092`) | PIR non-membership at h1, `unstirred lookup` | `evidence/2026-10-16-lookup-real-testnet.txt` |
| M1 step 4 (**uncommitted**) | proof package schema, the identity and spending-key path, bundle, prove, sign, `unstirred prove` | — (synthetic only: `evidence/2026-10-17-gate-m1-step4.md`) |

Step 4 is in the working tree, not committed (it was built under an explicit "do not commit").
`git status --short` lists it; the new files are `crates/unstirred-core/src/package.rs`,
`crates/unstirred-prover/src/{identity,spend_authority,bundle,prove}.rs`,
`crates/unstirred-cli/src/prove.rs` and `crates/unstirred-prover/tests/{prove,identity}.rs`.

`unstirred-verifier` and `unstirred-gate` are still one-line doc stubs. A real K=12 delegation
proof is now built, verified, signed and packaged in `tests/prove.rs`, over a synthetic wallet and
a synthetic nullifier set. Nothing in this repo has yet proved anything about real coins.

Last gate run: `evidence/2026-10-17-gate-m1-step4.md` — `GATE: PARTIAL`, exit 0, 41 tests passed,
0 failed; fmt and clippy clean. Covers A03, A04, A07, A12–A24, plus the prover halves of A02 and
A06. Open: A01, A05, A08–A11, the verifier half of A02/A06, and every live-service path.

## 2. What to do next, precisely

Two things close M1, in this order.

**(a) `unstirred verify`** — the smallest honest version: PRD §6 steps 3, 5, 7, 8 with the roots
passed in, so a package can be accepted in a fresh process. Everything it needs exists:

- `unstirred_core::ProofPackage::parse` → `instance()`, `proof_bytes()`, `session_pubkey()`,
  `challenge()`, `binding_signature()` (all strict, all `PROOF_INVALID` on a malformed field)
- `derive_round_id` for step 3, `rk_from_coordinates` + `binding_message` for step 7,
  `voting_circuits::delegation::verify_delegation_proof` for step 8
- the full root authority (two agreeing lightwalletd servers), the marker store and the challenge
  store are M2/M3, not this step

**(b) the real-testnet proof.** `unstirred prove` has never touched the holder's wallet. Running it
needs the age identity (`../unstirred.age`), the local `nf-server` from §3 below and a
lightwalletd. It decrypts the real mnemonic, so it is the holder's call to run, and the run must
be recorded in `evidence/` with heights, roots and commit before README claims it.

Then: register any new acceptance IDs in `briefing/PRD.md` §10 **before** writing their tests, with
a `DECISIONS.md` row — that is the standing rule in this repo.

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
  `DECISIONS.md` and §5 below. It was running during the step 3 run as:
  ```
  cd /home/kingghidorah/Documents/projects/bounties/zecathon/vote-nullifier-pir/nf-server
  ../target/release/nf-server serve --zcash-network test \
      --pir-data-dir ../pir-data/test-v0123 --port 3000 --pir-poly-len 4096
  ```
  Dataset `pir-data/test-v0123`: ironwood, dataset version 2, height 4,506,510, 276,626 ranges,
  depth 19 (12+7), poly_len 4096. Its `circuit_root` is the expected h1 root
  `8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26`.
  `pir-data/test` holds an older export at the same height whose roots happen to match.
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

`lookup` and `prove` need the nf-server above running. `prove` refuses to overwrite an existing
`--out` file. Nothing above has been run for `prove` yet: that command line is what the next
session should run and record, not a recorded run. `unstirred codes` lists the closed code set.

## 5. Traps that already cost time

`briefing/BUILD_NOTES.md` is the full list and should be read before touching the PIR, witness or
proving paths. The ones most likely to bite the next step:

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
- One real K=12 proof costs about 70 s in `tests/prove.rs`, including Halo2 key generation on
  first use. All three tests in that file share one proof on purpose.

## 6. Open items

- **Step 4 is uncommitted.** Committing it is the first thing to decide. The gate passes from the
  working tree; `evidence/2026-10-17-gate-m1-step4.md` records that run and the tree it ran on.
- `scripts/gate.sh` still prints `GATE: PARTIAL`: the leak scanner (A02) and the claims check
  (every README claim links to `evidence/`) are not gate steps yet. A02's substance now exists as
  a test over the proof package (`tests/prove.rs`), but the gate does not scan logs or the repo.
- `unstirred verify` and the verifier crate: not built. `unstirred-gate`: not built.
- The M1 session key is a fresh 32 random bytes with no private half (see `DECISIONS.md`); M3 has
  to replace it with a session the gate issues, and issue the challenge instead of the holder.
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
