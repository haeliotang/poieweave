# Joint Coverage Certificate — pre-registration

- Experiment ID: `certificate-s2-joint`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: model-level coverage certificate for Base-06.

## Question

Can the independent-vs-joint coverage counterexample be packaged as a
machine-checkable certificate?

## Method

Base-06, 0/1/2 failures, 12 h horizon, 200 kWh / 10 kW battery. Certificate
contains model SHA-256, total, independent 25, joint 24, gap 1 and the
`pump-p1,pump-p2` gap pair. Python verifies hash and all counts.

Prediction: `certificate=ok`.

## Limitation

Model-level coverage only; not a joint survival schedule certificate.
