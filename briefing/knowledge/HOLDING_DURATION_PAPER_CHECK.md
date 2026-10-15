# Paper check: proof of holding duration on Valar's delegation proof

Date: 2026-10-15
Status: PASS on paper. Not yet run in code. Not yet reviewed by a cryptographer.
Sources read (code, not docs): valargroup/voting-circuits (src/delegation/README.md, builder.rs, prove.rs),
valargroup/vote-nullifier-pir (README, nf-server), ZcashFoundation/zebra (zebra-rpc/src/methods.rs z_gettreestate).

## Test 1: soundness of pinning the two roots to different heights (H0 < H1)
The delegation circuit takes nc_root and nf_imt_root as independent public inputs; nothing in-circuit ties them to one height.
- Condition 10: each real note has a Merkle path to nc_root. Pinned to Ironwood root at H0 → note commitment existed at or before H0.
- Condition 13: each real note's true nullifier is NOT in the IMT. Pinned to Ironwood nullifier set at H1 → note not spent at any height <= H1 (set is append-only).
- Together: note existed by H0 and was unspent through H1 = held continuously over [H0, H1].
- Double use: gov_null = Poseidon(nk, dom, real_nf), dom = f(context id). Independent of H0/H1, so the same note cannot be reused in one context with a different window. Context id must NOT include heights.
Verifier obligations: enforce H0 < H1; take both roots from its own node, never from the prover; H1 deep enough to avoid reorgs.
Deviation: Valar's docs require both roots at the same height. That is their voting protocol rule, not a circuit constraint. Our argument above must be reviewed (ask Valar / ZODL engineers).

## Test 2: what the nullifier tree contains
vote-nullifier-pir syncs Ironwood nullifiers from lightwalletd up to SYNC_HEIGHT (multiple of 10), builds the canonical K=2 punctured-range IMT, and serves it over YPIR.
`nf-server verify-root` independently rebuilds the root from raw blocks → verifier can authenticate nf_imt_root at H1.
Circuit condition 12 derives the note's TRUE nullifier; condition 13 proves it is outside that set. PASS.

## Test 3: historical Ironwood roots from Zebra
z_gettreestate(height) returns the Ironwood tree and root for any height at or after NU6.3 (AnyChainIronwoodTree). PASS.

## New engineering risks found
- Witness at H0: prover needs each note's Merkle path against the H0 root, not the current one. Wallet checkpoint retention may not keep H0; rebuild from Ironwood commitments since activation (tree is young, so cheap; compact-block download reveals nothing).
- Spend-authority binding: proof needs only FVK-level secrets. Verifier must also check a RedPallas signature under rk over (context, session key, challenge). Also prevents front-running a seen proof.
- Wallets move notes (change, consolidation, ZIP 318 migration). Any move resets the clock for that value. UX must say "held without moving".

## Limits to state
- Minimum 0.125 ZEC (circuit enforces >= 1 ballot). Max 5 notes per proof.
- Duration capped by Ironwood age (activation 2026-07-28, block 3,428,143).
- Window (H0, H1) fixed per context, otherwise the chosen H0 leaks note age.
- Point-in-time: says nothing about after H1.

## Prior art check
Snapshot proofs exist (hanh Proof of Balance 2024: "no proof of age"; Eiger zair Namada airdrop toolkit; Valar voting). No holding-duration proof found as of 2026-10-15.
