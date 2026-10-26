# Releasing

Everything an owner does by hand, in order, with the reason each step is by hand. Nothing in this
repository tags, publishes or uploads anything: the release workflow builds binaries for a tag that
**already exists**, and `cargo publish` is never run by CI.

Read this top to bottom before the first step. Two of these steps cannot be undone.

---

## Before anything

```
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 ./scripts/gate.sh
```

`GATE: PASS`, from a clean tree, on the commit you are about to tag. The gate is fmt, clippy with
`-D warnings`, the whole test suite, the API reference with no rustdoc warning, the leak scan, the
claims check and the README status table against `CLAIMS.md`.

Then switch on the one repository setting the site needs, if it is not on yet:

> **Settings → Pages → Build and deployment → Source = "GitHub Actions"**

Nothing else — no branch, no directory. The `pages` workflow runs on every push to `master`, and
`Cargo.toml`'s `homepage` and `documentation` point at that site, so this wants to be on before the
crates go to crates.io.

While you are in Settings, the social preview image:

> **Settings → General → Social preview → Edit → Upload an image**

Upload `assets/icon.png`. It is already 1280×640, which is the size GitHub asks for, so it needs no
cropping. That image is what Twitter, Slack, Discord and GitHub itself show when somebody pastes a
link to the repository; without it they show a generic grey box.

---

## 1. Tag v0.1.0

```
git tag -a v0.1.0 -m "Unstirred 0.1.0"
git push origin v0.1.0
```

Pushing the tag starts `.github/workflows/release.yml`, which builds `unstirred` for
`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and `x86_64-apple-darwin` and attaches each
tarball with its `.sha256` to the release. It creates no tag of its own and publishes nothing to
crates.io.

Wait for all three jobs to be green before the next step. README's 60-second path tells a stranger
to download from the releases page, so that page must have assets on it.

**Until this tag is pushed, four download links in the repository do not resolve.** README's step 1
and `docs/GETTING_STARTED.md`'s install block name `v0.1.0` tarballs, and `docs/INTEGRATION.md`
takes the crates by `tag = "v0.1.0"`. They name the release this step creates, not the earlier
`v0.1.0-rc.1`, so this step is what makes them work. Nothing else in the repository has to change
when it does.

## 2. Write the release notes

Paste the text under [Release notes for v0.1.0](#release-notes-for-v010) below into the release's
body, on the releases page, and publish it as the latest release.

## 3. Attach the walkthrough video

```
gh release upload v0.1.0 video/out/unstirred-demo.mp4 --clobber
```

Check what you are about to upload first — the file is 16 MB and the recorded one has a known
digest:

```
sha256sum video/out/unstirred-demo.mp4
# 9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399
```

Once it is attached, its URL is
`https://github.com/DrApkFile/Unstirred/releases/download/v0.1.0/unstirred-demo.mp4`. Put that in
the one place that wants it: the `PLACEHOLDER` comment under **Watch the demo** in `README.md`.
Rebuilding the site with `VIDEO_URL=<that url> scripts/build-site.sh` puts it on the landing page
too.

## 4. Publish to crates.io

**This cannot be undone.** A published version can be yanked but never replaced, and the five
crates depend on each other by exact version, so a mistake in the first one has to be worked around
in the next four. The order is the one cargo chose for itself in
[the publish dry run](../evidence/2026-10-23-publish-dry-run.md):

```
cargo publish -p unstirred-core
cargo publish -p unstirred-prover
cargo publish -p unstirred-verifier
cargo publish -p unstirred-gate
cargo publish -p unstirred
```

Each one waits for the index to carry the previous crate, because every internal dependency is
pinned `=0.1.0`. If cargo says a dependency is not on crates.io yet, wait a minute and run that one
again; do not add `--no-verify`.

Two things to know before the first command:

- **The names are not reserved.** If any of the five is taken, stop: the whole set has to be
  renamed together, and that is a different release.
- **A dry run of all five is already recorded** — every crate packs and compiles from its own
  tarball, with no file from `evidence/`, `briefing/` or any test directory in any of them. Re-run
  it on the tagged commit if you want to see it again:

```
cargo publish --dry-run --workspace
```

---

## Release notes for v0.1.0

Everything between the rules is the body to paste.

---

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove to a gate — a community, an allowlist, a faucet — that they
held at least 0.125 ZEC in the Ironwood pool, **unmoved**, from one block to another, without
revealing their address, their balance, which coins, or anything they have ever spent. Each gate
admits the same coins exactly once, and two gates cannot tell they saw the same holder.

The note-commitment root is pinned at `h0` and the nullifier-set root at `h1`. Pinning the two at
**different** heights is what turns a snapshot proof into a proof about a span of time — a balance
you had for a month, not a balance you borrowed for a block.

**Try it in 60 seconds, with no wallet and no node.** Download the binary for your platform below,
then re-verify a real proof made from a real Zcash testnet wallet against the live testnet root:

```
unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest examples/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

### What has actually been run against the Zcash testnet

Every one of these is a recorded run in `evidence/`, cited from `CLAIMS.md` with the heights, the
commit and the command. Nothing below is a description of something that would work.

- A real holding-duration proof and its binding signature, made from a real testnet wallet over
  **h0 4,505,790 through h1 4,506,510**, and ADMITTED in a fresh process against the real testnet
  roots.
- **Two independent root sources agreeing** on the h0 root — zec.rocks lightwalletd and the owner's
  own zebrad 7.0.0-rc.0, both reporting `a168…692d` — with the governing tip taken as the lower of
  the two.
- The same coins **refused at the gate that already admitted them** (`ALREADY_USED_IN_CONTEXT`,
  nothing written) and **admitted at a second gate** with a marker set sharing nothing with the
  first's.
- A **second wallet, funded after h0**, refused by the room's own rule on the holder's side
  (`COINS_TOO_NEW`) with no proof made and nothing sent to the gate; and its forced submission
  refused by a live gate on the root (`ROOT_MISMATCH`) with nothing written.
- The holder's **wallet file unchanged** by all of it — same sha256 before and after.
- The recorded proof **still ADMITTED today**, against a root fetched from the live chain about
  38,000 blocks past h1, by a program that uses nothing but the library's public API.

### Limits, stated

- The **h1 nullifier-set root is operator-pinned.** The verifier cannot yet check it for itself —
  nf-server `verify-root` is not wired in — and every output that shows that root says
  `operator-pinned`.
- **Testnet only.** Nothing here has been exercised against mainnet.
- The **threshold is fixed at 0.125 ZEC**, because it is one ballot in the circuit, and a proof
  covers at most five notes.
- An application **cannot yet bind its own data** — a vote, a payout address — into the signed
  message, so a verdict says "this holder qualifies" and not "this holder chose that".
- A single root source is possible and **labels itself as unchecked** in every line of output.

`SPEC.md` §10 and `briefing/LEAKS.md` are the full lists. `CLAIMS.md` marks anything not built as
not built.

### Credits

Built on [Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits) —
`voting-circuits 0.12.2`, audited, used **unchanged** and version-pinned — together with their
nullifier PIR server and client, which make the non-membership lookup private. The wallet stack is
`zcash_client_sqlite` and `zcash_client_backend`. The gate, the verifier, the command line, the
duration statement and the single-use markers are this project's.

MIT OR Apache-2.0.

---
