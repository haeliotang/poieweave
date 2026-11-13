# S4 Asset+Lot Mutations — pre-registration

- Experiment ID: `s4-asset-lot-mutations`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one pre-failed asset + one removed accepted lot.

## Question

Does Rust/independent agreement survive a combined equipment failure and
inventory loss, not just one or the other?

## Method

For each of 9 Base-05 assets and 8 accepted lots, mark the asset failed in the
TOML, remove the lot, then compare all 9 target verdicts at 24 h between Rust
and Python: 72 models, 648 cases.

Prediction: agreement; mismatches investigated.

## Limitation

One asset + one lot; no multi-asset or process combinations.
