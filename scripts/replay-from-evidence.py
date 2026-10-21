#!/usr/bin/env python3
"""Builds the Holders' Room page's replay acts from the recorded two-room knock.

Every value in crates/unstirred-gate/ui/replay/*.json comes from
evidence/2026-10-20-two-room-knock.txt, so the demo video is rendered from the real run rather
than from numbers somebody typed. Re-running this script rewrites the four acts; the gate's A44
test then checks that each one still matches the state contract and that the words for every code
they name are `unstirred-core`'s own.

The one value the run does not hold is the moment of the refusal: the store writes nothing for a
refused knock. It is derived from that knock's recorded challenge expiry less the store's
600-second TTL, which is the second the challenge was issued, and every act that shows it says so
on the page.

Usage: python3 -I scripts/replay-from-evidence.py
"""

import json
import pathlib
import re
import sys

CHALLENGE_TTL_SECONDS = 600
ROOM_LOG_LIMIT = 50
MEMBERS_AT_FIRST_PAINT = 1000
RECORDED_RUN = "evidence/2026-10-20-two-room-knock.txt"
HEADER = "evidence/2026-10-20-two-room-knock.meta.md"
OUT = pathlib.Path("crates/unstirred-gate/ui/replay")

ALREADY_USED = {
    "ALREADY_USED_IN_CONTEXT": {
        "kind": "REFUSED",
        "message": "These coins have already been used at this gate. Nothing was admitted.",
        "retryable": False,
    }
}

REPLAYED_FROM = (
    "Replay of the two-room knock recorded on the real Zcash testnet on 2026-10-20: "
    f"{RECORDED_RUN} (header: {HEADER})."
)


def recorded():
    text = pathlib.Path(RECORDED_RUN).read_text()
    objects = [json.loads(line) for line in text.splitlines() if line.startswith("{")]
    listing = objects[0]
    logs = {body["context"]: body for body in objects[1:]}
    expiries = re.findall(r"expires (\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ)", text)
    if len(listing["rooms"]) != 2 or len(logs) != 2 or len(expiries) != 3:
        sys.exit(f"{RECORDED_RUN} is not the run this script reads: {len(logs)} logs, {len(expiries)} challenges")
    return listing, logs, expiries


def room_state(described, log, markers, verdicts, recent):
    return {
        "context": described["context"],
        "network": described["network"],
        "window": {"h0": described["window"]["h0"], "h1": described["window"]["h1"]},
        "min_zatoshis": described["min_zatoshis"],
        "h1_root": described["h1_root"],
        "h1_root_provenance": described["h1_root_provenance"],
        "limit": ROOM_LOG_LIMIT,
        "markers": [{"marker": row["marker"], "admitted_at": row["admitted_at"]} for row in markers],
        "verdicts": [
            {"verdict": row["verdict"], "code": row["code"], "at": row["at"], "at_note": None}
            for row in verdicts
        ],
        "recent": recent,
    }


def act(name, room, rooms, listing, codes, provenance):
    return name, {
        "mode": "replay",
        "source": "real-testnet",
        "replay": name,
        "room": room,
        "members": MEMBERS_AT_FIRST_PAINT,
        "gate": {"root_sources": listing["root_sources"], "note": listing.get("note")},
        "codes": codes,
        "rooms": rooms,
        "provenance": [REPLAYED_FROM] + provenance,
    }


def main():
    listing, logs, expiries = recorded()
    first, second = listing["rooms"][0], listing["rooms"][1]
    first_log, second_log = logs[first["context"]], logs[second["context"]]
    refused_at = iso_to_unix(expiries[1]) - CHALLENGE_TTL_SECONDS
    admitted_first = first_log["verdicts"][0]
    admitted_second = second_log["verdicts"][0]

    def empty(described, log):
        return room_state(described, log, [], [], [])

    def full(described, log, recent):
        return room_state(described, log, log["markers"], log["verdicts"], recent)

    as_recent = lambda row: {"verdict": row["verdict"], "code": row["code"], "at": row["at"], "at_note": None}
    refusal = {
        "verdict": "REFUSED",
        "code": "ALREADY_USED_IN_CONTEXT",
        "at": refused_at,
        "at_note": (
            "time derived: the second this knock's challenge was issued. A refusal is never written down, "
            "so the run holds no time of its own for it."
        ),
    }

    acts = [
        act(
            "act1-rule",
            first["context"],
            [empty(first, first_log), empty(second, second_log)],
            listing,
            {},
            [
                "The rule, the window, the threshold, the root sources and the h1 root are that run's own. "
                "Nothing had been admitted at this point in it."
            ],
        ),
        act(
            "act2-admitted",
            first["context"],
            [full(first, first_log, [as_recent(admitted_first)]), empty(second, second_log)],
            listing,
            {},
            [
                "The first knock was admitted. These five markers and this verdict are what the room's own log "
                "showed afterwards."
            ],
        ),
        act(
            "act3-refused",
            first["context"],
            [
                full(first, first_log, [refusal, as_recent(admitted_first)]),
                empty(second, second_log),
            ],
            listing,
            ALREADY_USED,
            [
                "The same coins were knocked at this room a second time and were refused. The store wrote nothing "
                "for it: the five markers and the one admission above are unchanged.",
                f"The refusal is shown at the second its challenge was issued ({expiries[1]} less the store's "
                f"{CHALLENGE_TTL_SECONDS}-second TTL). A refusal is never written down, so the run holds no "
                "timestamp of its own for it.",
            ],
        ),
        act(
            "act5-second-room",
            second["context"],
            [
                full(first, first_log, [as_recent(admitted_first)]),
                full(second, second_log, [as_recent(admitted_second)]),
            ],
            listing,
            {},
            [
                "The same coins were admitted at a second room. Each room's log showed five markers, and the two "
                "sets have none in common."
            ],
        ),
    ]

    for name, state in acts:
        path = OUT / f"{name}.json"
        path.write_text(json.dumps(state, indent=2) + "\n")
        print(f"{path}: {len(state['rooms'])} rooms, {sum(len(r['markers']) for r in state['rooms'])} markers")


def iso_to_unix(stamp):
    import datetime

    return int(datetime.datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp())


if __name__ == "__main__":
    main()
