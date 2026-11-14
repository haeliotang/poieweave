# S4 Independent Certificate — pre-registration

- Experiment ID: `certificate-s4-independent`
- Date: 2026-11-14
- Status: completed; 12th bundle layer
- Scope: aggregate the energy, PV, grease, concurrency, time and approval-schedule and tool-cycle independent-model checks.

## Question

Can the narrow independent-model results be packaged as one bundle layer whose
counts are checked against the actual expected files?

## Method

Emit checks for energy/PV/grease with case and matched counts. The Python
verifier re-reads each expected file, recomputes counts, and requires all
matched.

Prediction: `certificate=ok`, bundle remains green.

## Limitation

Aggregates narrow models; does not run them or prove whole-system equivalence.
