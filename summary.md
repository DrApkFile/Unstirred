The live sequence matched. holders-room was admitted, the same package was refused, and the same coins were admitted again under `second-room` with a different set of markers. Nothing was committed.

Zebra's `getblockcount` was 1,836,400, still short of h0 4,505,790, so both admits used one lightwalletd source. Each one printed `agreement: not checked (one source)`. The wallet file's sha256 was the same before and after (`73005f5547b3eb3ade60e6ffdd71170726485e9ae953961f15066fc71ae4629a`). The run is in `evidence/2026-10-17-admit-two-rooms.txt`. The database is `/tmp/unstirred-gate.sqlite`.

| Step | Result |
|---|---|
| `gate challenge` for holders-room | `c5891c15509bd8886521cc752619239d0af80e0db29ac97b13760fbe4448fc00` |
| prove, then admit | exit 0, `ADMITTED` |
| admit that same package | exit 1, `ALREADY_USED_IN_CONTEXT` |
| `gate challenge` for second-room | `4dc13019109e3af6d95a4f953457df5022cc3b34e7529015610685d01591635f` |
| prove, then admit | exit 0, `ADMITTED` |
| `gate markers` | 5 markers in each context, no hex in common; two `ADMITTED` rows; no challenge hashes |

`second-room` was the holders-room manifest with the context changed, same window, written only to `/tmp/unstirred-second-room.json`. h1 root `8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26`. nf-server was the v0.12.3 binary on port 3000, and it was stopped after the proofs.

```bash
DB=/tmp/unstirred-gate.sqlite
M=/home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json
S=/tmp/unstirred-second-room.json
W=/home/kingghidorah/Documents/projects/bounties/zecathon/wallet
H1=8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
CHAL=$(./target/release/unstirred gate challenge --db "$DB" --manifest "$M")
./target/release/unstirred prove --wallet "$W" --manifest "$M" \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://127.0.0.1:3000 \
    --identity /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age \
    --challenge "$CHAL" --expected-h1-root "$H1" --out /tmp/unstirred-holders.json
./target/release/unstirred admit --db "$DB" --package /tmp/unstirred-holders.json \
    --manifest "$M" --root-source lwd:https://testnet.zec.rocks:443 \
    --single-root-source --h1-root "$H1"
# same admit command again → exit 1, ALREADY_USED_IN_CONTEXT
CHAL2=$(./target/release/unstirred gate challenge --db "$DB" --manifest "$S")
./target/release/unstirred prove --wallet "$W" --manifest "$S" \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://127.0.0.1:3000 \
    --identity /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age \
    --challenge "$CHAL2" --expected-h1-root "$H1" --out /tmp/unstirred-second.json
./target/release/unstirred admit --db "$DB" --package /tmp/unstirred-second.json \
    --manifest "$S" --root-source lwd:https://testnet.zec.rocks:443 \
    --single-root-source --h1-root "$H1"
./target/release/unstirred gate markers --db "$DB"
```

`./scripts/gate.sh` finished `GATE: PARTIAL`, exit 0 (`evidence/2026-10-17-gate-m2-step2.md`). The store tests A02b, A05, A05c, A05d, A07, A08 and the invalid-proof no-write check passed there. A one-source `verify` or `roots` run now says `agreement: not checked (one source)` in the human output and in JSON. `HANDOFF.md`, `briefing/BUILD_NOTES.md`, the README status rows and the L4, L8 and L9 rows in `briefing/LEAKS.md` are updated.
