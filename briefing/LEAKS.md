# LEAKS: Unstirred

Rubric: "Leaks are disqualifying, not deductions." Every channel below has a defence with a test, or an explicit non-defence. Status: planned (tests to be written in M2–M5).

| # | Channel | What could leak | Defence | Test | Status |
|---|---|---|---|---|---|
| L1 | Proof instance | Address, balance, notes | Zero-knowledge; public inputs are roots, round id, rk, van_comm, cmx_new, markers, nf_signed | A02 | planned |
| L2 | Exact balance | Weight | Hidden in van_comm; no weight published | A02 | planned |
| L3 | Coin age | When coins arrived | Window fixed by the gate for everyone | A12 | planned |
| L4 | Cross-gate linking | Same holder at two gates | Markers derived per context | A08 | planned |
| L5 | Nullifier lookup | Which nullifier is ours | YPIR private query; no plaintext fallback, and an unreachable server is PIR_UNAVAILABLE | A15 | **tested on synthetic data** (evidence/2026-10-16-gate-m1-step3.md) |
| L5b | Lookup answer | The bracket around our nullifier | `nf_bounds` and the path stay private witnesses: `lookup` prints the h1 root, tree positions and yes/no only, and `NoteNonMembership`'s Debug hides both | A02 | planned |
| L6 | H0 witness rebuild | Note positions | Built locally from wallet shards plus public frontier; no targeted query | A02 | planned |
| L7 | rk reuse | Linking two admissions | Fresh alpha per proof | A08b | planned |
| L8 | Replay / front-running | Someone else uses my proof | Challenge + session binding | A07 | planned |
| L9 | Gate logs | Correlation | Store markers and verdicts only; no IP, no package bodies | A02b | planned |
| L10 | Front-end | Analytics, third-party fonts/scripts | No analytics; no third-party requests | A02c | planned |
| L11 | Wallet history | Proving moves coins | Proving creates no transaction; wallet opened read-only | A16 | planned |
| L12 | Network / IP | Holder's IP at submission | **Non-defence.** Recommend Tor; stated in README | — | stated |
| L13 | Timing | Admission time | **Non-defence**, not identifying; stated | — | stated |
| L14 | Root servers | They see a public height | Not user data; stated | — | stated |
| L15 | Spend-key exposure | Signing reveals key | RedPallas signature under randomized key; key never leaves the prover | A06 | planned |
