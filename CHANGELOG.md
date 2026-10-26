# Changelog

Every release, and what changed in it. Dates are the date of the tagged commit.

This file records what the software does, not how it was built: a decision that changed a frozen
rule is in [`DECISIONS.md`](DECISIONS.md), and a claim about what has actually been run is in
[`CLAIMS.md`](CLAIMS.md) with the file that settles it.

## 0.1.0 — 2026-10-26

The first release. Everything below is new.

### What it does

- **Proves a holding, not a snapshot.** A Zcash holder proves to a gate that they held at least
  0.125 ZEC in the Ironwood pool, unmoved, from one block height through another, without
  revealing their address, their balance, which coins, or anything they have ever spent. The
  note-commitment root is pinned at `h0` and the nullifier-set root at `h1`, which is what makes
  the statement about a span of time rather than a moment ([`SPEC.md`](SPEC.md) §3).
- **One admission per holder per gate.** Each proof writes five markers, scoped to the gate's own
  context, in one SQLite transaction with the challenge it answered. The same coins are refused at
  that gate afterwards (`ALREADY_USED_IN_CONTEXT`) and admitted at a different gate with a marker
  set that shares nothing with the first.
- **A proof alone is not enough.** The holder also signs the statement, their session key and the
  gate's challenge under `rk`, rebuilt by the verifier from the instance's own coordinates. A
  viewing key can produce a valid proof; it cannot produce that signature.
- **Roots the verifier establishes itself.** Two independent sources — lightwalletd gRPC and a
  Zebra node's JSON-RPC — must agree on the `h0` root, and the governing tip is the lower of the
  two. One source is an explicit opt-in that labels itself as unchecked in every line of output.
  Nothing is ever read from the package the holder submitted.
- **Unknown is never success.** Sixteen verdict codes in one closed set, three outcomes
  (ADMITTED / REFUSED / INCONCLUSIVE), and a source that cannot be reached is inconclusive rather
  than a statement about the holder.
- **A gate that serves its own pages.** `unstirred gate serve` answers five JSON routes and serves
  the Holders' Room page and a room picker, assembled in-process. No page asks any other origin
  for anything: no analytics, fonts, scripts, beacons or images.
- **A private nullifier lookup.** Every nullifier is queried through Valar's PIR client and only
  ever leaves the machine inside a YPIR ciphertext. There is no plaintext path.
- **A read-only holder.** The prover opens the wallet read-only and nothing in this project can
  build or broadcast a transaction.

### Crates

`unstirred-core`, `unstirred-prover`, `unstirred-verifier`, `unstirred-gate`, and the `unstirred`
command line. Every public item is documented, and the examples in those docs are compiled.

### Known limits

Stated rather than discovered: the `h1` nullifier-set root is **operator-pinned** and labelled as
such everywhere it appears, because nf-server `verify-root` is not wired in yet; the threshold is
fixed at 0.125 ZEC by the circuit; an application cannot yet bind its own data into the signed
message; and nothing here has been exercised against mainnet. [`SPEC.md`](SPEC.md) §10 and
[`briefing/LEAKS.md`](briefing/LEAKS.md) are the full lists.

### Credits

Built on [Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits)
(`voting-circuits 0.12.2`, audited, used unchanged and version-pinned) and their nullifier PIR
server and client. The Zcash wallet stack is `zcash_client_sqlite` and `zcash_client_backend`.
