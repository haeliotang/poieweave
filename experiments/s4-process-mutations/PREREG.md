# S4 Process Mutations — pre-registration

- Experiment ID: `s4-process-mutations`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one-process removal mutations of Base-05.

## Question

Do the Rust and independent verdicts agree when a process is removed from the
model, not just a lot or asset?

## Method

Remove each of the 7 Base-05 processes in turn, run all-asset verdicts at 24 h
through Rust and the Python recursive checker, and compare.

Prediction: agreement; differences would be model-semantics findings.

## Limitation

One process at a time; no source corruption, multi-removal or timing changes.
