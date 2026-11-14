# S4 Degraded+Lot Mutations — pre-registration

- Experiment ID: `s4-degraded-lot`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one degraded asset + one removed accepted lot.

## Question

Do Rust and Python agree on combined partial equipment degradation and
inventory loss?

## Method

9 assets × 8 accepted lots = 72 models; set asset degraded, remove lot, compare
all 9 target verdicts: 648 cases.

Prediction: agreement; mismatches investigated.

## Limitation

One asset + one lot.
