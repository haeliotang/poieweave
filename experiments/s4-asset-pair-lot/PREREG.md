# S4 Asset-Pair+Lot Mutations — pre-registration

- Experiment ID: `s4-asset-pair-lot`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: two failed assets + one removed accepted lot.

## Question

Does Rust/independent agreement survive two equipment failures plus an
inventory loss in the same model?

## Method

Enumerate 36 asset pairs × 8 accepted lots = 288 mutated models. Compare all 9
target verdicts at 24 h between Rust and Python: 2592 cases.

Prediction: agreement; mismatches investigated.

## Limitation

Two assets + one lot; no process combinations.
