# S4 Degraded Status Mutations — pre-registration

- Experiment ID: `s4-degraded`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: set each Base-05 asset to `degraded`.

## Question

Do Rust and Python agree on the non-failed `degraded` status, not only
`available`/`failed`?

## Method

Set each of the 9 assets to `degraded` in turn, run all 9 target verdicts at
24 h, compare: 81 cases.

Prediction: agreement; mismatches investigated.

## Limitation

One asset at a time; Base-05 only.
