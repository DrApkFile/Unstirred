# FAILURE_MODES: Unstirred

## Lightwalletd (H0 tree state; wallet sync)
| Mode | Can it happen? | Guard chosen | Where | Test |
|---|---|---|---|---|
| Server lies about root | Yes | Two independent servers; exactly one agreeing answer or INCONCLUSIVE | verifier/roots | A10 |
| Timeout / unreachable | Yes | ROOT_SOURCE_UNAVAILABLE (retryable); never admit | verifier/roots | A10b |
| Retry | Yes | Retry reads only (idempotent); bounded backoff | verifier/roots | — |
| Server sees which height we ask | Yes | Height is public (from the manifest); no user data in the query | — | LEAKS row |
| Wallet sync reveals interest | Partly | Compact-block sync downloads all blocks; prover never fetches a specific txid | prover | A02 |

## nf-server / PIR (H1 nullifier set)
| Mode | Can it happen? | Guard chosen | Where | Test |
|---|---|---|---|---|
| Served tree differs from chain | Yes | Gate runs `nf-server verify-root` at start; refuses to start on mismatch | gate startup | A09b |
| PIR server down | Yes | Prover refuses with PIR_UNAVAILABLE; no plaintext fallback query | prover | A15 |
| Lookup answer not authentic | Yes | Prover checks the returned path against the pinned H1 root before proving | prover | A15b |
| Sync height not a multiple of 10 | Yes | Manifest rule h1 % 10 == 0 | core | A13 |

## Halo2 proving
| Mode | Can it happen? | Guard chosen | Where | Test |
|---|---|---|---|---|
| Proof generation fails | Yes | Typed error; no package written | prover | — |
| Slow on judge hardware | Yes | Release build only; README states measured time with hardware | docs | — |

## Gate storage (challenges, markers)
| Mode | Can it happen? | Guard chosen | Where | Test |
|---|---|---|---|---|
| Same package twice concurrently | Yes | Challenge consume and marker insert in one transaction; UNIQUE constraints | verifier/store | A05c (race test) |
| Crash between steps | Yes | Single transaction; no partial admission | verifier/store | A05d |
| Challenge replay | Yes | Single use + expiry | verifier/store | A07 |

## Chain reorg
| Mode | Can it happen? | Guard chosen | Where | Test |
|---|---|---|---|---|
| H1 reorged after sync | Low | WINDOW_INVALID unless h1 <= tip - FINALITY_DEPTH (100) | core | A13b |

## Network upgrade (NU7 on mainnet 2026-11-05)
| Mode | Can it happen? | Guard chosen | Where | Test |
|---|---|---|---|---|
| Crate cohort cannot parse post-NU7 blocks | Possible on mainnet | Demo pinned to testnet (already NU7); recorded evidence kept | docs | — |
