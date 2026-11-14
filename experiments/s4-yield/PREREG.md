# S4 Yield Mutations — pre-registration

- Experiment ID: `s4-yield`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: add a 0.5 process yield to each Base-05 process.

## Question

Do Rust and Python agree once both apply process yield?

## Method

For each of the 7 processes, add a 0.9 and a 0.5 yield on its first output
with a matching reject resource; compare all 9 target verdicts: 126 cases.

Prediction: agreement; mismatch investigated.

## Limitation

One process at a time; yield 0.5.
