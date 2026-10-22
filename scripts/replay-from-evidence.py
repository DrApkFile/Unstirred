#!/usr/bin/env python3
"""Builds the Holders' Room page's replay acts from the recorded runs.

Every value in crates/unstirred-gate/ui/replay/*.json comes from a file in evidence/, so the demo
video is rendered from real runs rather than from numbers somebody typed. Acts 1, 2, 3 and 5 come
from evidence/2026-10-20-two-room-knock.txt; act 4, the newcomer, comes from the two Bob runs of
2026-10-21 and is only written once the forced knock has been recorded. Re-running this script
rewrites every act it can build, and writes the list of them beside the act files as acts.rs, so
an act cannot exist without being served. The gate's A44 test then checks that each one still
matches the state contract and that the words for every code they name are `unstirred-core`'s own.

The one value a run does not hold is the moment of a refusal: the store writes nothing for a
refused knock. It is derived from that knock's recorded challenge expiry less the store's
600-second TTL, which is the second the challenge was issued, and every verdict that shows it
carries an at_note saying so, which the page prints in its provenance foot.

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
EVIDENCE = pathlib.Path("evidence")
# The newcomer's two runs are found by name rather than written out here, so a second recording of
# either one is picked up without this script being edited: the newest file wins.
BOB_REFUSED_BY_THE_RULE = "*bob-knock.txt"
BOB_FORCED = "*bob-forced-knock.txt"

ALREADY_USED = {
    "ALREADY_USED_IN_CONTEXT": {
        "kind": "REFUSED",
        "message": "These coins have already been used at this gate. Nothing was admitted.",
        "retryable": False,
    }
}

ROOT_MISMATCH = {
    "ROOT_MISMATCH": {
        "kind": "REFUSED",
        "message": (
            "The proof does not match this gate's window: the coins were not held unmoved from h0 "
            "through h1."
        ),
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


def bob():
    """The newcomer's two runs: the rule's own refusal, and the forced submission the gate refused.

    Returns None until the forced knock has been recorded, so the other acts can still be built
    before act 4 exists. Every number it reads is printed by `unstirred knock` itself.
    """
    refused = newest(BOB_REFUSED_BY_THE_RULE)
    forced = newest(BOB_FORCED)
    if forced is None:
        return None
    by_the_rule = refused.read_text() if refused else ""
    text = forced.read_text()
    if "ROOT_MISMATCH" not in text or "REFUSED" not in text:
        sys.exit(f"{forced} holds no REFUSED / ROOT_MISMATCH verdict: the forced knock is what act 4 shows")
    expiries = re.findall(r"expires (\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ)", text)
    first_mined = re.search(r"first mined at block (\d+)", text)
    unspent_through = re.search(r"unspent through block (\d+)", text)
    proved_against = re.search(r"appear in, at block (\d+)", text)
    if not (expiries and first_mined and unspent_through and proved_against):
        sys.exit(
            f"{forced} does not carry the lines act 4 is built from: a challenge expiry, the "
            "coins' first mined block, the block they are unspent through, and the block whose root "
            "the forced proof was built against"
        )
    room = re.search(r"^room\s+(\S+)", text, re.MULTILINE)
    return {
        "room": room.group(1) if room else "holders-room",
        "at": iso_to_unix(expiries[0]) - CHALLENGE_TTL_SECONDS,
        "expiry": expiries[0],
        "from": int(first_mined.group(1)),
        "to": int(unspent_through.group(1)),
        "root_at": int(proved_against.group(1)),
        "forced_run": str(forced),
        "rule_run": str(refused) if refused and "COINS_TOO_NEW" in by_the_rule else None,
    }


def newest(pattern):
    """The most recent recording of one run, by filename. Every evidence file is dated, so sorting
    by name is sorting by date."""
    found = sorted(EVIDENCE.glob(pattern))
    return found[-1] if found else None


def room_state(described, log, markers, verdicts, recent, holder=None):
    return {
        "context": described["context"],
        "network": described["network"],
        "window": {"h0": described["window"]["h0"], "h1": described["window"]["h1"]},
        "min_zatoshis": described["min_zatoshis"],
        "h1_root": described["h1_root"],
        "h1_root_provenance": described["h1_root_provenance"],
        "limit": ROOM_LOG_LIMIT,
        # A holder line illustrates one act against the rule. Only act 4 draws one, and its two
        # heights are read from the holder's own wallet by the knock that was recorded: the
        # two-room run holds no holder's block range, only its verdicts and markers.
        "holder": holder,
        "markers": [{"marker": row["marker"], "admitted_at": row["admitted_at"]} for row in markers],
        "verdicts": [
            {"verdict": row["verdict"], "code": row["code"], "at": row["at"], "at_note": None}
            for row in verdicts
        ],
        "recent": recent,
    }


def act(name, room, rooms, listing, codes, provenance, replayed_from=None):
    return name, {
        "mode": "replay",
        "source": "real-testnet",
        "replay": name,
        "room": room,
        # Presentation mode is a way of reading an act, not part of it: the gate sets it from
        # ?present=1 on whichever act is asked for.
        "present": False,
        "members": MEMBERS_AT_FIRST_PAINT,
        "gate": {"root_sources": listing["root_sources"], "note": listing.get("note")},
        "codes": codes,
        "rooms": rooms,
        "provenance": [replayed_from or REPLAYED_FROM] + provenance,
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

    def full(described, log, recent, holder=None):
        return room_state(described, log, log["markers"], log["verdicts"], recent, holder)

    as_recent = lambda row: {"verdict": row["verdict"], "code": row["code"], "at": row["at"], "at_note": None}
    refusal = {
        "verdict": "REFUSED",
        "code": "ALREADY_USED_IN_CONTEXT",
        "at": refused_at,
        "at_note": derived_time_note(expiries[1]),
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
                "for it: the five markers and the one admission above are unchanged."
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

    newcomer = bob()
    if newcomer:
        acts.insert(3, newcomer_act(newcomer, first, first_log, second, second_log, listing, admitted_first))
    else:
        print(f"act4-newcomer: not written, no evidence/{BOB_FORCED} has been recorded yet")

    for name, state in acts:
        path = OUT / f"{name}.json"
        path.write_text(json.dumps(state, indent=2) + "\n")
        print(f"{path}: {len(state['rooms'])} rooms, {sum(len(r['markers']) for r in state['rooms'])} markers")
    write_registry([name for name, _ in acts])


def newcomer_act(newcomer, first, first_log, second, second_log, listing, admitted_first):
    """PRD section 9's act 4: the gate refuses a holder whose coins are newer than its h0, on the
    roots it pinned itself. The verdict and the holder's own line come from the forced knock; the
    room around them is the two-room run, which is what the page says on itself."""
    if newcomer["room"] != first["context"]:
        sys.exit(
            f"{newcomer['forced_run']} knocked on {newcomer['room']!r}, but the recorded room is "
            f"{first['context']!r}: act 4 can only be built from a run against that room"
        )
    refusal = {
        "verdict": "REFUSED",
        "code": "ROOT_MISMATCH",
        "at": newcomer["at"],
        "at_note": derived_time_note(newcomer["expiry"]),
    }
    holder = {
        "label": "the newcomer's coins, from the holder's own wallet (the gate never sees this)",
        "from": newcomer["from"],
        "to": newcomer["to"],
    }
    provenance = [
        f"The room's five markers and its earlier admission are the two-room knock of 2026-10-20: "
        f"{RECORDED_RUN} (header: {HEADER}).",
        f"The proof was built against the latest note-commitment root these coins appear in, at block "
        f"{newcomer['root_at']:,}, which is not this room's h0. The gate decides from the roots it pinned, "
        f"so it refused the package without reading what the package claims.",
        "The holder's line is read from the holder's own wallet by that knock and is never sent: the gate "
        "is told no height and no value of theirs.",
    ]
    if newcomer["rule_run"]:
        provenance.append(
            f"Before that, the room's own rule refused these coins COINS_TOO_NEW on the holder's side and no "
            f"proof was made: {newcomer['rule_run']}."
        )
    as_recent = lambda row: {"verdict": row["verdict"], "code": row["code"], "at": row["at"], "at_note": None}
    return act(
        "act4-newcomer",
        newcomer["room"],
        [
            room_state(
                first,
                first_log,
                first_log["markers"],
                first_log["verdicts"],
                [refusal, as_recent(admitted_first)],
                holder,
            ),
            room_state(second, second_log, [], [], []),
        ],
        listing,
        ROOT_MISMATCH,
        provenance,
        replayed_from=(
            f"Replay of the newcomer's forced knock, recorded against the real Zcash testnet: "
            f"{newcomer['forced_run']}. The gate answered REFUSED / ROOT_MISMATCH."
        ),
    )


def write_registry(names):
    """The acts and the list of them are one output: an act file cannot exist without being served,
    and a name here cannot point at a file that was never generated."""
    entries = "".join(f'    ("{name}", include_str!("{name}.json")),\n' for name in names)
    path = OUT / "acts.rs"
    path.write_text(
        "// Generated by scripts/replay-from-evidence.py beside the act files it lists. Do not edit by\n"
        "// hand: re-run the script instead, which writes both from the recorded runs in evidence/.\n"
        "\n"
        "/// Every act, in demo order: the name a URL asks for, and the state it renders.\n"
        f"pub const REPLAY_ACTS: [(&str, &str); {len(names)}] = [\n{entries}];\n"
    )
    print(f"{path}: {len(names)} acts")


def derived_time_note(expiry):
    """A refusal is never written down, so the only second the run holds for it is the one its
    challenge was issued at. The page prints this beside the act's provenance, and marks the time
    itself as derived, so a derived number is never shown as an observed one."""
    return (
        f"Time derived: the second this knock's challenge was issued ({expiry} less the store's "
        f"{CHALLENGE_TTL_SECONDS}-second TTL). A refusal is never written down, so the run holds no "
        "timestamp of its own for it."
    )


def iso_to_unix(stamp):
    import datetime

    return int(datetime.datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp())


if __name__ == "__main__":
    main()
