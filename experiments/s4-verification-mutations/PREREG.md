# S4 Verification Mutations — pre-registration

- Experiment ID: `s4-verification-mutations`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: downgrade each Base-05 process verification to `declared`.

## Question

Do Rust and Python agree when a process becomes under-verified, rather than
missing?

## Method

Downgrade each of the 7 processes in turn, run all 9 target verdicts at 24 h,
and compare: 63 cases.

Prediction: agreement; mismatches are reason-semantics findings.

## Limitation

One process at a time; no capability-map changes.
