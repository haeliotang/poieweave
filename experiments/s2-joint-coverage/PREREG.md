# S2 Joint Coverage — pre-registration

- Experiment ID: `s2-joint-coverage`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-06 model-level comparison; no physical claim.

## Question

Across all 0/1/2-asset failures on Base-06, what is the difference between the
independent recovery envelope and a joint policy using one shared inventory and
one serial repair order?

## Method

Enumerate the 56 cases (1 + 10 + 45 pairs). For each independently recoverable
case, schedule its failures at t=0, run the multi-service greedy policy against
the shared sleeve/bronze inventory, and mark joint recovery only if no repair is
blocked. Compare counts and list the independent-but-not-joint gaps.

## Prediction

Base-06 has two pumps with one sleeve and insufficient bronze. Independent
coverage should include the `pump-p1,pump-p2` pair; the joint policy should not.
Expected: independent 25/56, joint 24/56, optimistic gap ≥1.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

Single serial policy, immediate repair at t=0, no resupply or regeneration, and
synthetic quantities.
