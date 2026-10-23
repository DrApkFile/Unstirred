# crates.io publish dry run: the whole workspace, 0.1.0

What this is: `cargo publish --dry-run --workspace`, which packs each crate into the tarball
crates.io would receive and then **compiles each one from that tarball**, in dependency order.
Nothing was uploaded: every crate ends "aborting upload due to dry run". **Nothing is published,
and no crates.io token was used.**

Commit `3f79c4e` plus the uncommitted release-prep tree (HANDOFF §11): the CLI package renamed
`unstirred-cli` → `unstirred`, every crate carrying `description`, `readme`, `include`,
`keywords`, `categories`, `repository`, `homepage` and `documentation`, and every internal
dependency carrying `version = "=0.1.0"` beside its path.
Date: 2026-10-22T23:59:05Z to 2026-10-23T00:13:55Z (14m 50s).
Toolchain: `cargo 1.91.1 (ea2d97820 2025-10-10)`, `rustc 1.91.1 (ed61e7d7e 2025-11-07)`.
Command: `cargo publish --dry-run --workspace --allow-dirty` (`--allow-dirty` because the tree is
not committed yet; the real publish runs from the tagged commit with a clean tree).
Exit: 0.

## The run

```
   Packaging unstirred-core v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-core)
   Packaging unstirred-prover v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-prover)
   Packaging unstirred-verifier v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-verifier)
   Packaging unstirred-gate v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-gate)
   Packaging unstirred v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-cli)
   Verifying unstirred-core v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-core)
   Verifying unstirred-prover v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-prover)
   Verifying unstirred-verifier v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-verifier)
   Verifying unstirred-gate v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-gate)
   Verifying unstirred v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-cli)
   Uploading unstirred-core v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-core)
warning: aborting upload due to dry run
   Uploading unstirred-prover v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-prover)
warning: aborting upload due to dry run
   Uploading unstirred-verifier v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-verifier)
warning: aborting upload due to dry run
   Uploading unstirred-gate v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-gate)
warning: aborting upload due to dry run
   Uploading unstirred v0.1.0 (/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/crates/unstirred-cli)
warning: aborting upload due to dry run
```

Cargo chose that order itself, and it is the order the five crates must be published in:
**unstirred-core, unstirred-prover, unstirred-verifier, unstirred-gate, unstirred.**

## What each crate would ship

`cargo package --list`, same tree. No file from `evidence/` or `briefing/` appears in any of them,
and no test directory does either, because every crate's `include` names only its own sources,
its README, and `ui/**/*` for the gate, which the gate compiles in with `include_str!`.

| Crate | Files | From `evidence/`, `briefing/` or `tests/` |
|---|---|---|
| `unstirred-core` | 14 | 0 |
| `unstirred-prover` | 27 | 0 |
| `unstirred-verifier` | 24 | 0 |
| `unstirred-gate` | 42 (12 of them `ui/`) | 0 |
| `unstirred` | 28 | 0 |

## What this run does not prove

- That the names are free on crates.io. They were checked free on 2026-10-15 (`DECISIONS.md`) and
  nothing has reserved them since as far as this run knows: a dry run never contacts the registry
  about a name.
- That the published crates work for anyone else. The verify step compiles each tarball against
  this machine's resolved dependency versions and this toolchain.
- Anything about the Zcash network. No wallet, no node, no nf-server and no chain were involved.
