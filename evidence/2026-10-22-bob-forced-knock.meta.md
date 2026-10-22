# Header for `2026-10-22-bob-knock.txt` and `2026-10-22-bob-forced-knock.txt`

The recorded output of these two runs is exactly what the terminal held: neither file carries a
header of its own. This file is that header, written beside them so the recorded output stays
untouched. Nothing here was produced by the runs; it is what the runs were. They are the demo's
act 4 (PRD §9): the newcomer whose coins are newer than this room's h0, refused first by the room's
own rule on the holder's side, then — forced — by the gate itself, on the roots it pinned.

| | |
|---|---|
| Date | 2026-10-22; the forced knock's challenge was issued at 05:59:43Z (it expires 06:09:43Z, ten minutes later) |
| Network | `test` (Zcash testnet) |
| Window | h0 4,505,790 through h1 4,506,510 |
| Threshold | 12,500,000 zatoshis |
| Binary | `target/release/unstirred`, built from the uncommitted M4 step 1 tree whose gate run is [evidence/2026-10-22-gate-m4-step1.md](2026-10-22-gate-m4-step1.md) |
| Gate | `http://127.0.0.1:8080`, the same `gate.toml` and the same database as the two-room knock of 2026-10-20, so `holders-room` already held that run's five markers and its one admission |
| Rooms | `holders-room` and `second-room`, both written by `unstirred gate init` over the same window |
| Root sources | `lwd:https://testnet.zec.rocks:443` and `zebra:http://127.0.0.1:18232` — two, both live, so no answer carried the single-source label |
| h1 root | `8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26`, operator-pinned (nf-server `verify-root` is deferred to P1) |
| Wallet | `/home/kingghidorah/Documents/projects/bounties/zecathon/bob-wallet`, the newcomer's — a second testnet wallet funded after h0 — opened read-only on both knocks; no transaction was created |
| PIR | nf-server (`v0.12.3`, dataset `pir-data/test-v0123`, height 4,506,510) on `http://localhost:3000` |

## What the two files hold

`2026-10-22-bob-knock.txt` is an ordinary knock. The room's own rule refused it
`COINS_TOO_NEW` — "the account held no unspent Ironwood note at h0" — and the run ends with
"nothing was sent to the gate" and "no challenge was asked for, so this gate has no record of this
attempt". There is no challenge line and no "proving now" line in it, because neither happened:
that is the order A45 holds, and it is what this run was recorded to show. The run of 2026-10-21
(`2026-10-21-bob-knock.txt`) is the same refusal under the old order, kept as recorded.

`2026-10-22-bob-forced-knock.txt` is the same wallet and the same room with
`--force-unqualified-demo`. It announces itself, names the code the room refused it with, and says
what it is proving instead: one note, first mined at block 4,526,842 and still unspent at block
4,528,029, proved against the note-commitment root at 4,528,029 — 21,519 blocks after this room's
h0. Those two heights are read from the holder's own wallet and printed on the holder's own
terminal; the package carries the room's published window and no height or value of the holder's.
The gate answered **REFUSED / ROOT_MISMATCH**, "The proof does not match this gate's window: the
coins were not held unmoved from h0 through h1", retryable false. The last line of the file is
`GET /rooms/holders-room/log` afterwards: the same five markers and the same one `ADMITTED` as
before, stamped `1792506814` (2026-10-20T14:33:34Z), so the refusal wrote nothing.

The package itself is a real proof — real notes, a real witness at a real root, a real private
nullifier lookup at h1 and a real binding signature. Nothing in it is faked; the only thing that
does not match is the one root the gate decides from.

## The knocks, as they were run

```bash
cd /home/kingghidorah/Documents/projects/bounties/zecathon
U=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred
B=/home/kingghidorah/Documents/projects/bounties/zecathon/bob-wallet
ID=<the age identity for that wallet>
H1=8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26

$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$B" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # COINS_TOO_NEW, nothing sent, exit 1

$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$B" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1" --force-unqualified-demo   # ROOT_MISMATCH, exit 1

curl -s http://127.0.0.1:8080/rooms/holders-room/log
```

The gate was started as in `evidence/2026-10-20-two-room-knock.meta.md`, with nf-server up as in
`HANDOFF.md` §3.
