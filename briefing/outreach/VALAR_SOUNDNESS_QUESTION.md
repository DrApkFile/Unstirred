Title: Question on reusing the delegation proof with split-height roots (proof of holding duration)

Hi Valar team, and thanks for publishing voting-circuits, vote-nullifier-pir and zcash_voting as reusable infrastructure.

I'm building a small hackathon tool (Zecathon, Core & Tooling) that would reuse the delegation proof (ZKP #1)
outside voting, as an anonymous "held since" credential. All credit to your circuits; no circuit changes planned.

The idea: the verifier pins nc_root to the Ironwood note commitment root at height H0 and nf_imt_root to the
Ironwood nullifier IMT root at a later height H1 (H0 < H1), both taken from its own Zebra node / verify-root.

Our reasoning:
- Condition 10 (Merkle path to nc_root@H0) shows each real note existed at or before H0.
- Condition 13 (true nullifier outside the IMT@H1) shows it was not spent at any height up to H1, since the set is append-only.
- So each note was held unspent over [H0, H1].
- Condition 14's gov nullifier depends only on (nk, dom(round_id), real_nf), so with round_id = a context id that
  excludes heights, the same note cannot be reused in the same context with a different window.

Your docs say both roots must come from the same snapshot height. Is that a requirement of the circuit's soundness,
or of the voting protocol only? Is there anything in the IMT construction or in the circuit we are missing that
would break the split-height statement?

We will also verify a RedPallas signature under rk over (context, session key, challenge), since the proof alone
needs only FVK-level secrets.

Happy to share the repo and spec. Thank you!
