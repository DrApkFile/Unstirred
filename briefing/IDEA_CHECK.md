# IDEA_CHECK: Unstirred

Date: 2026-10-15
Track: Core & Tooling (fallback framing: Wildcard)
Status: GO, conditional on REALITY_SPIKES.md S1-S8 passing by 2026-10-17 evening.

## Hook
Unstirred: proof your coins stayed put. Nothing else.
A holder proves "I have held at least 0.125 shielded ZEC, unmoved, since block H0" for one context,
revealing no address, no balance, no transaction. Nothing is broadcast to the chain.

## Who, the job, how they cope today, where it fails
- Gate owners: a holder community, an allowlist (e.g. the organiser's SNARKLIST), a faucet or airdrop operator, a holders-only poll.
- Today: manual review, KYC, public-chain role bots (expose the wallet), or snapshot balance proofs.
- Failure: snapshot proofs are gamed by buying or borrowing right before the snapshot; public bots dox members; KYC kills privacy.
  Community asked for "proof of age" in 2024 (forum thread 46626); none of the existing tools provide it.

## Hard core
1. Holding-duration statement built on Valar's audited delegation circuit, with the note-commitment root pinned
   to H0 and the nullifier-set root pinned to H1 (see knowledge/HOLDING_DURATION_PAPER_CHECK.md).
2. Verifier that authenticates both roots itself (Zebra z_gettreestate for H0, nf-server verify-root for H1),
   never from the prover.
3. Context-scoped one-use markers (gov nullifiers) and cross-context unlinkability.
4. Spend-authority binding: RedPallas signature under the proof's rk over (context, session key, challenge).
5. Prover-side witness reconstruction at H0.

## Thin-wrapper test
No model call anywhere. Deterministic Rust. Passes.

## Dependency deletion test
| Removed | What breaks | Critical path? |
|---|---|---|
| Zcash Ironwood notes | Nothing to prove | Yes |
| Valar voting-circuits | Would need a new circuit (not feasible in window) | Yes, credited openly |
| Valar PIR / nf-server | Non-membership query would leak which nullifier | Yes |
| Zebra | Verifier cannot authenticate H0 root | Yes |

## Boundary
Prover proposes a proof. Verifier code decides. Verifier never accepts a root, height or context from the prover.
Refusals are named: COINS_TOO_NEW, ALREADY_USED_IN_CONTEXT, BAD_SIGNATURE, ROOT_MISMATCH, WINDOW_INVALID, PROOF_INVALID.

## The loop
gate owner publishes manifest (context, H0, H1, network) → holder proves → verifier accepts or refuses with a named code
→ reuse in same context refused → same coins in another context accepted and unlinkable → verifier stores only markers.
Step others skip: the newcomer refusal (COINS_TOO_NEW) and the cross-context unlinkability check.

## What we deliberately do not claim
- Proof of personhood (it is stake-and-time weighted: one credential per 0.125 ZEC held).
- Any threshold other than 0.125 ZEC (fixed by circuit); no weighting.
- Anything about holdings after H1.
- Network-level anonymity of the prover.
- Post-quantum security.
- Mobile wallet support (v1 is CLI / desktop).
- That Valar endorses the split-height use (unless they reply).

## Leak table (fatal gate)
| Channel | Risk | Defence or stated non-defence |
|---|---|---|
| Address, balance, notes | Revealed by proof | Zero-knowledge; only markers, rk, roots, context public |
| Exact balance | Bucketed weight | van_comm hides it; v1 publishes no weight |
| Coin age | User-chosen H0 would leak it | Window fixed per context by gate owner |
| Cross-context linking | Same coins in two gates | Marker = Poseidon(nk, dom(context), real_nf); differs per context |
| Nullifier lookup | Server learns which nullifier | YPIR private query (Valar) |
| H0 witness rebuild | Server learns note positions | Download all Ironwood commitments; no targeted query |
| Proof replay / front-running | Seen proof reused | Signature binds proof to holder's session key and fresh challenge |
| Verifier logs | Correlation | Store markers only; no IP logging in our service |
| Network / IP | Correlation at submission | Stated non-defence; recommend Tor |
| Timing | Proving right after window opens | Stated; window fixed per gate |

## Rubric → proof map
| Criterion | Answer | Proof a judge can open |
|---|---|---|
| Privacy (fatal) | Leak table, each row tested or stated | LEAKS.md, tests A02-* |
| Usefulness | One-call gate for any app; organiser's allowlist | Holders' Room demo, verifier API |
| Execution | Real proofs on real Ironwood notes | evidence/ proofs + roots + heights, clean-clone verify script |
| Originality | First private proof of holding duration on Zcash | LANDSCAPE.md prior-art table, commit history in window |

## Consent / third parties
Holders opt in by proving. Gate owners publish their own manifests. No third-party data.

## Verdict
GO if S1-S8 pass. Fallback on failure: idea 4 (cryptographically bounded agent wallets, FROST policy co-signer).
