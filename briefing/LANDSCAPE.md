# LANDSCAPE

Researched: 2026-10-15
Note: Zecathon submissions are sealed, so neighbours below are prior Zcash entries and live products.

## House style in recent Zcash events
Small, real, cryptographically honest tools. Winners use one Zcash-native primitive deeply
(FROST, viewing keys, encrypted memos, RedPallas signatures, PCZT) and ship a working demo.
Wallet clones and dashboards are common; simulated transactions lose.

## Neighbours
| Project / product | What it does | Our positioning line |
|---|---|---|
| Zodl (ZODL, ex-Zashi) | Shielded-by-default mobile wallet, NEAR-Intents swaps, CrossPay, Vault (desktop, Oct 7) | We build on the shielded rail they popularised; we don't compete with a wallet |
| NEAR Intents | Cross-chain solver network, ~$29B cumulative volume | Exit leg is a trust and compliance boundary (see $589k hold, Sept 2026) |
| Login with ZcashMe | Pay-to-login via encrypted memo + OIDC | Prior art for memo-based protocols |
| zk.poker | FROST 2-of-3 shielded escrow for a game | Prior art for FROST escrow |
| CYZE | FROST threshold wallet | Prior art for threshold custody |
| Pendrake Watch | UFVK watch-only wallet | Prior art for viewing-key disclosure |
| zkSNARKs blind auction | Organiser's own sealed-bid uniform-price sale | Private Markets judges will know auctions well; be better, not similar |
| t2z projects (Zypherpunk) | Transparent-to-shielded tooling | Shielding UX is a known ECC interest |

## The gap
Almost every Zcash app is private on-chain and leaky everywhere else: transparent hops on
entry/exit, deposit addresses, amount and timing round-trips, light-wallet server queries,
analytics, logs. The rubric makes any leak fatal, and almost nobody ships a leak audit.

## Prepared judge questions
- Q: Where could this leak? A: `LEAKS.md`, channel by channel, each with a test or a stated non-defence.
- Q: Is this Ironwood-correct? A: Yes, txs write Ironwood outputs; tx IDs in `evidence/`.
- Q: Does it run? A: Clean-clone script, real tx IDs.

## Prior art already built (do not repeat) — researched 2026-10-15
| Idea | Already done by |
|---|---|
| Sealed-bid uniform-price auction | Crafts (Arcium track winner, Solana Privacy Hack, Feb 2026) |
| Private OTC desk, private prediction market | Undesk, Epoch (Solana Privacy Hack) |
| Private Venmo / Wise payments | Veil (Aztec track best overall) |
| Private payroll | "Private Payrolls" (Solana Privacy Hack) |
| Wallet privacy/exposure scanner | Solana Privacy Scanner, solana-exposure-scanner |
| Payment disclosure / receipts (ZIP 311) incl. Ironwood | zreceipt, zcash-disclosure |
| Private light-wallet tx fetch | zcash-prb (private receive bridge); Zakura/Vizor wallet-pir |
| Agent payments with ZEC / x402 | ZAP1 + x402, CipherPay x402, zshield-mcp, ZecAgent, zaygent |
| FROST escrow / threshold wallet | zk.poker, CYZE |
| Shielded login | ZcashMe, ZecAuth |
| Mesh / offline broadcast | pcash wallet (Meshtastic), btcmesh |
| Shielded coinholder voting | Valar Group (live, NU7 vote Aug-Sep 2026) |
| Private proof of balance for voting and airdrops | hanh, "Coin Voting / Shielded Airdrop / Proof of Balance" (ZCG application Jan 2024; Coin Voting 2.0 app used in 2025 dev-fund vote). No proof of age. |
| Public holding-duration record on Bitcoin | PATINA (bitcoinuniverseio/patina, Aug 2026): tracks how long a Bitcoin output stays unspent, depth tiers. Public, no ZK. |
| "Proof of tenure" for Sybil resistance | Patina (Vana, patinadata.xyz, Jul 2026): account-history age as anti-Sybil evidence. Tagline: "Anyone can make a new account. Nobody can make an old one." Not crypto holdings, not ZK on chain. |
| Tenure-based DeFi | tenure-hook (Uniswap v4, on-chain history gates depth), tenure-weighted-fees, TenureLP (2026). Public. |
| Shielded airdrop to ZEC holders | Namada ZEC-NAM shielded airdrop protocol (2023-2025) |

Calibration: the Zcash builder scene is saturated with small, AI-assisted tools. "First to build X" claims need checking every time.
