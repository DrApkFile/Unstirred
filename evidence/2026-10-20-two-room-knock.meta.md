# Header for `2026-10-20-two-room-knock.txt`

The recorded output of this run is `evidence/2026-10-20-two-room-knock.txt` and it is exactly what
the terminal held: it carries no header of its own. This file is that header, added afterwards as a
separate file on 2026-10-21 so the recorded output stays untouched. Nothing here was produced by
the run; it is what the run was.

| | |
|---|---|
| Date | 2026-10-20 |
| Network | `test` (Zcash testnet) |
| Window | h0 4,505,790 through h1 4,506,510 |
| Threshold | 12,500,000 zatoshis |
| Binary | `target/release/unstirred`, built with `cargo build --release` from the M3 step 1 tree later committed as `6c2a288` |
| Rooms | `holders-room` and `second-room`, both written by `unstirred gate init` over the same window |
| Root sources | `lwd:https://testnet.zec.rocks:443` and `zebra:http://127.0.0.1:18232` — two, both live, so no answer carried the single-source label |
| h1 root | `8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26`, operator-pinned (nf-server `verify-root` is deferred to P1) |
| Wallet | `/home/kingghidorah/Documents/projects/bounties/zecathon/wallet`, opened read-only on every knock; no transaction was created |
| PIR | nf-server (`v0.12.3`, dataset `pir-data/test-v0123`) on `http://localhost:3000` |

The `.txt` holds, in order: the `GET /rooms` body, the three `unstirred knock` outputs
(`holders-room` ADMITTED, the same coins at `holders-room` REFUSED `ALREADY_USED_IN_CONTEXT`,
`second-room` ADMITTED), then the two `GET /rooms/<context>/log` bodies — five markers in each
room, no hex in common, one `ADMITTED` verdict each and no challenge hash. The marker rows are
stamped `1792506814` (2026-10-20T14:33:34Z) for `holders-room` and `1792509475`
(2026-10-20T15:17:55Z) for `second-room`.

## The gate, as it was started

```bash
cd /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred
cargo build --release
cd /home/kingghidorah/Documents/projects/bounties/zecathon
U=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred
mkdir -p gate && cd gate
$U gate init --network test --context holders-room --h0 4505790 --h1 4506510 --out holders-room.json
$U gate init --network test --context second-room  --h0 4505790 --h1 4506510 --out second-room.json
cat > gate.toml <<'TOML'
listen = "127.0.0.1:8080"
database = "gate.sqlite"
root_sources = ["lwd:https://testnet.zec.rocks:443", "zebra:http://127.0.0.1:18232"]
h1_root = "8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26"

[[rooms]]
manifest = "holders-room.json"

[[rooms]]
manifest = "second-room.json"
TOML
$U gate serve --config gate.toml
```

Both rooms end at h1 4,506,510, so one `h1_root` covers both.

## The knocks, as they were run

```bash
cd /home/kingghidorah/Documents/projects/bounties/zecathon
U=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred
W=/home/kingghidorah/Documents/projects/bounties/zecathon/wallet
ID=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age
H1=8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26

curl -s http://127.0.0.1:8080/rooms
$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # ADMITTED, exit 0
$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # ALREADY_USED_IN_CONTEXT, exit 1
$U knock --gate http://127.0.0.1:8080 --room second-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # ADMITTED, different markers
curl -s http://127.0.0.1:8080/rooms/holders-room/log
curl -s http://127.0.0.1:8080/rooms/second-room/log
```
