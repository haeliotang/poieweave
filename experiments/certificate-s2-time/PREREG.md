# S2 Time Certificate — pre-registration

- Experiment ID: `certificate-s2-time`
- Date: 2026-11-14
- Status: completed; 13th bundle layer
- Scope: dense/sparse horizons, deadline flip, grease/insert boundaries.

## Question

Can the time-semantics results be packaged as a self-contained certificate whose
invariants an independent verifier can check?

## Method

Emit dense event times, sparse horizons, deadline arithmetic, grease/insert
boundaries, the regeneration negative control, and the long-repair regeneration variant. The Python verifier checks the arithmetic and saturation invariants.

Prediction: `certificate=ok` and bundle remains green.

## Limitation

Certificates record experiment outputs; the verifier checks invariants, not the
full simulator.
