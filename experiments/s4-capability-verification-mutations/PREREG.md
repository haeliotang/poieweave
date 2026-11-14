# S4 Capability Verification Mutations — pre-registration

- Experiment ID: `s4-capability-verification-mutations`
- Date: 2026-11-14
- Status: completed; found and fixed a Rust semantic gap
- Scope: downgrade each Base-05 capability-verification entry to `declared`.

## Question

Do Rust and Python agree when an asset's capability evidence is downgraded,
even though its process evidence is unchanged?

## Method

Downgrade each of the 5 capability-verification entries in turn, run all 9
target verdicts at 24 h, and compare.

Prediction: agreement. A mismatch is treated as a semantic finding.

## Limitation

One capability at a time; Base-05 only.
