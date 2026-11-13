# S4 Asset-Pair Mutations — pre-registration

- Experiment ID: `s4-asset-pairs`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: pairs of pre-failed assets in Base-05.

## Question

Does Rust/independent agreement survive two simultaneous pre-failed assets?

## Method

Enumerate C(9,2)=36 asset pairs, mark both failed in the TOML, compare all 9
target verdicts at 24 h: 324 cases.

Prediction: agreement; mismatches investigated.

## Limitation

Two assets only; no inventory/process combinations.
