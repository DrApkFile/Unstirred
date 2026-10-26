# Clean-clone audit, as a stranger — 2026-10-26

Both paths README offers, followed exactly, on a machine that holds none of this project's state:
the **release download** and the **build from source**. Then the 60-second verify, the two
commands README offers beside it, and the exported replay site opened in a browser.

- **Machine**: Linux 7.2.9-arch1-1, Intel Core i3 M 370 at 2.40 GHz, **4 cores**, 7 GB of memory.
- **Toolchain**: `cargo 1.91.1 (ea2d97820 2025-10-10)`, `rustc 1.91.1 (ed61e7d7e 2025-11-07)`,
  which is what `rust-toolchain.toml` pins.
- **Commit under audit**: `456d19a` plus the uncommitted final-audit part B working tree. Part B
  is not committed, so a real `git clone` could not see it; the tree was built instead with
  `git ls-files -co --exclude-standard | tar -T - -cf -`, which holds **exactly** what a clone of
  the next commit will hold: **342 files, 6.6 MB**, unpacked into
  `~/.cache/unstirred-clean-clone-2026-10-26/tree`.
- **Not touched**: neither wallet. Nothing here opens a wallet, builds a transaction or knocks a
  gate. Every command below is a verifier reading a recorded package.

---

## Path 1: the release download

README step 1 says to take the tarball for your platform from the releases page, check it against
the `.sha256` beside it and unpack it. The latest release at the time of this audit is
**v0.1.0-rc.1**.

```
$ sha256sum -c unstirred-v0.1.0-rc.1-x86_64-unknown-linux-gnu.tar.gz.sha256
unstirred-v0.1.0-rc.1-x86_64-unknown-linux-gnu.tar.gz: OK
```

sha256 `3a4f58836e81c481eabf2bcd84196f36f476c1d3caee66997ff2adb0da8353c2`, 7,267,810 bytes.

```
$ ./unstirred --version
unstirred 0.1.0
```

**One fault found, and fixed.** The tarball holds exactly seven entries:

```
unstirred-v0.1.0-rc.1-x86_64-unknown-linux-gnu/{unstirred,README.md,SPEC.md,CLAIMS.md,LICENSE-MIT,LICENSE-APACHE}
```

— and **neither of the two files README's 60-second path names as arguments**:
`examples/holders-room.json` and `evidence/2026-10-17-first-proof.json`. A stranger who downloads
the binary has both commands and neither of their arguments, and nothing in the README tells them
to clone the repository as well. `.github/workflows/release.yml` now creates
`dist/<name>/examples` and `dist/<name>/evidence` and copies those two files (and `CHANGELOG.md`)
into every tarball, so the v0.1.0 assets carry them. The audit continued with the two files copied
in by hand, which is what the fixed workflow will produce.

**The manifest writes itself identically.** README step 2 says `unstirred gate init` writes the
same file the repository publishes:

```
$ unstirred gate init --network test --context holders-room --h0 4505790 --h1 4506510 --out holders-room.json
$ diff holders-room.json examples/holders-room.json
```

No output. Byte for byte identical.

**The 60-second verify**, exactly as README step 3 prints it:

```
root sources for the h0 Ironwood note-commitment root (network test, h0 4505790):
  lwd:https://testnet.zec.rocks:443  root a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d  tip 4545089
agreement: not checked (one source)
chain tip for the window check: 4545089 (the lower of the answers)
ROOT FROM ONE SOURCE: not independently checked
h1 nullifier-set root    8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
    pinned by the operator on the command line, labelled operator-pinned (nf-server verify-root is deferred to P1)

ROOT FROM ONE SOURCE: not independently checked
ADMITTED
verify does not record markers; use unstirred admit to enter a gate
```

**ADMITTED, exit 0.** The h0 root is
`a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` — the same root the runs of
2026-10-17 and 2026-10-20 reported for h0 4,505,790 — read at tip **4,545,089**, about 39,000
blocks past h1.

One note on the text this path reads: the rc.1 tarball carries the README **as it stood at that
tag** (2026-10-23), not the one in this working tree. The step-1 wording audited here is the
current one.

---

## Path 2: the build from source

README's build-from-source section, in the clean tree, with the registry already warm:

```
$ CARGO_BUILD_JOBS=2 cargo build --release --locked
   ...
    Finished `release` profile [optimized] target(s) in 27m 34s
```

**350 crates, 27 minutes 34 seconds, exit 0**, with two jobs on a four-core 2.40 GHz i3. `--locked`
means `Cargo.lock` was good enough to build from untouched. `target/` is 1.1 GB afterwards.
`docs/GETTING_STARTED.md` now quotes this run rather than an unrecorded figure.

```
$ ./target/release/unstirred --version
unstirred 0.1.0
```

**`gate init` again, from the built binary**: byte for byte identical to
`examples/holders-room.json`.

**The 60-second verify with the built binary**: **ADMITTED, exit 0, 44 seconds**, root
`a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` at tip **4,545,161**. README
says "about 35 seconds … on an idle machine", and this machine was not idle — the browser and this
session were on it.

**The two things README offers beside it**, both without a chain:

```
$ unstirred codes                       # 16 lines, the whole closed set, exit 0
$ unstirred verify --package <one byte changed> ...
REFUSED  PROOF_INVALID
  the delegation proof did not verify: delegation verification failed: ConstraintSystemFailure
  why:  The zero-knowledge proof did not verify. Nothing was admitted.
  next: Generate a fresh proof with an unmodified prover.
                                        # exit 1
```

The tampered package was the recorded one with a single hex digit of `proof` flipped (`e` → `0` at
offset 0). 35 seconds to refuse it.

**One rough edge, not fixed:** `unstirred codes | head -8` ends with Rust's standard
`failed printing to stdout: Broken pipe` panic after printing the eight lines asked for. It is
std's own behaviour on `SIGPIPE`, it loses no output and it affects no scripted use, so it was left
alone rather than changed hours before a release.

---

## The exported replay site, opened locally

From the clean tree, with nothing else running:

```
$ scripts/export-replay-site.sh ~/.cache/unstirred-clean-clone-2026-10-26/site
11 page(s) written; each one renders from the state it carries and fetches nothing
```

**11 files, 488 KB**: five acts, each in normal and presentation mode, and the room picker.

Checked over all eleven:

```
11 pages
off-origin references: none
broken local links: none
```

Then opened in Chromium (`file://`, 1280×900), three of them looked at:

- **the picker** — both rooms side by side, each card's "Open room" now opening that room's first
  recorded act rather than the gate route `/room/<context>` that has no file behind it here (A51).
- **act 2** — ADMITTED and the five markers.
- **act 4** — the newcomer's REFUSED / ROOT_MISMATCH, with the axis broken after h1 and
  "+20,332 blocks" on the mark past the break.

No gate, no chain, no wallet and no nullifier server was running for any of it.

---

## What this audit changed

| What | Where |
|---|---|
| The release tarballs carry `examples/holders-room.json`, `evidence/2026-10-17-first-proof.json` and `CHANGELOG.md`, so README's 60-second path runs from inside one | `.github/workflows/release.yml` |
| The build-from-source time is the one recorded here, not an unrecorded figure | `docs/GETTING_STARTED.md` |
| Both README verify examples name the same manifest path | `README.md` |
