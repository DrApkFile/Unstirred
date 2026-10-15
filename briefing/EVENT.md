# EVENT: ZECATHON (a Zcash privacy hackathon by zkSNARKs)

Researched: 2026-10-15
Sources: https://thezecathon.com/ (public page), organiser posts on X (@zksnarks_, 2026-09-28),
track and scoring text pasted from the signed-in site by the team on 2026-10-15.

## At a glance
| Item | Detail |
|---|---|
| Build window | 30 days, opened 2026-09-28 |
| Deadline | Oct 28, 2026. Time zone not stated. **Internal deadline: submitted by 2026-10-27 08:00 UTC** (safe for every time zone, including UTC+14). |
| Days left at time of writing | 13 |
| Judging | Submissions sealed until judging; judges not yet identified |
| Prizes | $100,000 total. Five tracks x $15,000. Grand prize $20,000 on top of a track prize. Community bounty (details truncated, confirm) |
| Entries | One submission per project, one account per project |
| Field size | Unknown |

## Scoring
No weights published. Four criteria, read literally:

| Criterion | Organiser wording (paraphrased) | Full marks looks like | Our planned proof |
|---|---|---|---|
| Privacy | Does it actually preserve privacy or only look like it? Leaks disqualify, they are not deductions | Zero channels that reveal who paid whom, on-chain or off | `LEAKS.md`: every channel enumerated (chain, memo, amount, timing, network, server logs, front-end analytics, deposit addresses) with a test or argument per channel |
| Usefulness | Would someone use it on Monday | A named user with a job this week | One real user story, run end-to-end in the demo |
| Execution | Does it run. Working beats ambitious | Judge reproduces it in minutes | Real mainnet (or clearly labelled testnet) tx IDs, clean-clone run script, 60-second judge path |
| Originality | Built for this hackathon, not repackaged | Commit history starting in the window | Fresh repo, behavioural commits, prior-work declaration |

Heaviest item: not stated. Treat Privacy as heaviest because it is the only one with a disqualifier.
**Fatal gate: any leak.** Consequence for us: claim narrowly. Every feature must pass the leak review before it ships; anything we cannot prove private is cut, not caveated.

## Eligibility gates (acceptance IDs)
| ID | Requirement | How we verify | Evidence file |
|---|---|---|---|
| A01 | Built on Zcash | Real shielded txs on chain | `evidence/A01-txids.md` |
| A02 | No privacy leak (fatal) | Leak checklist + adversarial tests | `LEAKS.md`, tests titled `A02 …` |
| A03 | It runs | Clean-clone script passes | `evidence/A03-clean-clone.log` |
| A04 | Built in the window | First commit >= 2026-09-28 | git log |
| A05 | Submitted before deadline (TZ TBC) | Submission receipt | `evidence/A05-receipt` |
| A06 | One track chosen | Submission form | — |

## Tracks
| Track | Organiser ask (paraphrased) | Notes |
|---|---|---|
| Shielded Payments | POS, payroll, remittance, subscriptions; money moves without publishing who paid whom | Most crowded track, expect many wallets |
| Private Markets | Dark-pool thesis: sealed-bid auctions, order flow, settlement; price public, participants private | Matches the organiser's own business (blind auction, Zilkroad marketplace) |
| Cross-Chain | Swaps, bridges, intents that reach other chains without unshielding on the way | Highest leak risk; deposit paths often transparent (verify) |
| Core & Tooling | Zebra, librustzcash, indexers, wallet SDKs, provers | Fewer entrants, judged by engineers |
| Wildcard | Art, games, social, hardware | Anything that makes a shielded tx worth making |

## Entry type chosen and why
Unstirred (private proof of holding duration). Track: Core & Tooling. It is a developer toolkit (crate, CLI, verifier service) with one real integration; fewer entrants, engineer judges, and the leak gate favours tight scope. See IDEA_CHECK.md.

## Stack and "real work" rules
Unknown beyond "on Zcash". Confirm: mainnet required or testnet OK? Hosted demo required?

### Stack facts that changed in 2026 (verify before building)
- Orchard is closed to new outputs; Ironwood (NU6.3, active since 2026-07-28, block 3,428,143) is the live shielded pool. Orchard-era code is wrong by default.
- zcashd halted at block 3,417,100 and does not support NU6.3. zcash.readthedocs.io describes zcashd: concepts only, not a runtime.
- Current stack: Zebra (node), Zaino (indexer / light-wallet server), Zallet (alpha wallet, breaking changes), librustzcash crates.
- NU7: live on public testnet since 2026-10-05 (25 s blocks, Sprout disabled, fee reserve). Mainnet decision 2026-10-20, target 2026-11-05, i.e. after our deadline. Testnet and mainnet rules differ during the build.
- Full stack detail: `knowledge/ZCASH-STACK.md` (primary-source briefing, 2026-10-14).

### Judging-window hazards (our demo must survive judging, not just submission)
- Zebra 6.4.x self-halts at its end-of-support height, approx 2026-11-02. Judging likely runs past that. Pin Zebra >= 7.0.0-rc.0 for anything a judge might run. Never 6.4.0 / 6.4.1.
- NU7 mainnet target 2026-11-05 also falls in the judging window. A mainnet demo built on NU6.3 rules may break mid-judging; public testnet is already on NU7 and stays valid. Leaning: develop on regtest, record and host the demo on testnet, keep recorded tx IDs as evidence.
- Crate cohort: stable `zcash_client_backend =0.24.0` / `zcash_client_sqlite =0.22.0` (Ironwood, NU6.3). NU7 needs the `-pre` cohort, which broke three times in 12 days. Decide per path via a reality spike, pin exact versions.

## What judges read, in order
Unknown. Assume: submission form, demo video, README, then repo. Write for that order.

## Submission fields → where our answer lives
Unknown. Copy field names here once seen.

## Rules on commits, history, prior work, updates during judging
Confirmed by the team from the site (2026-10-15): **no commits after the deadline.** Our rule: last commit before the internal deadline above; no pushes of any kind afterwards (docs included). "Originality: built for this hackathon": first commit in the window, Valar's crates credited as dependencies.

## Past winners and calibration (Zcash events, not this organiser)
- ZecHub Hackathon 3.0 (Aug 2026), DAO-judged on: works, meaningful Zcash use, completeness, originality, ecosystem impact, docs and ease of testing. Winners: ZEC-OS (explorer desktop), Login with ZcashMe (shielded-payment login via OIDC), zk.poker (mental poker + 2-of-3 FROST shielded escrow), CYZE (FROST threshold wallet), Pendrake Watch (UFVK watch-only wallet).
- Zypherpunk (Dec 2025): ECC prizes to two t2z (transparent-to-shielded) builds and Temi; Zolanear ShadeLink $5k.
- Pattern: real Zcash cryptography (FROST, RedPallas, viewing keys, PCZT, memos) plus something a judge can open. Counter-example from a 2026 entry: simulated tx hashes, no real broadcast.

## Organiser context
zkSNARKs is a 10,000-piece shielded-identity collection sold via a blind uniform-price auction (cleared at 1.5 ZEC). Public allegations (ZachXBT, 2026-09-19) that the team under-delivered. Allegations, not findings. Read prize-payment terms (currency, timing, KYC) before relying on the prize.

## Open questions for organisers
1. Deadline time zone and exact time on Oct 28.
2. Mainnet required, or is testnet acceptable?
3. Who judges, and is there a written leak definition (does network-level metadata count)?
4. Community bounty: what is it and how is it scored?
5. Commit rules during judging; prior-work declaration.
6. Team size limits; prize currency and payout terms.
