# S4 Pair-Lot Mutations — pre-registration

- Experiment ID: `s4-pair-mutations`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: all two-lot removals from Base-05.

## Question

Does Rust/independent verdict agreement survive simultaneous removal of two
accepted inventory items?

## Method

Enumerate all pairs of the 8 accepted lots (28 models). For each, run all 9
asset verdicts at 24 h through both implementations and compare.

Prediction: agreement; any mismatch is investigated.

## Limitation

Pairs only; no process+lot combined mutations or triples.
