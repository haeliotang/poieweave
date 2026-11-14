# S4 Approval Certificate — pre-registration

- Experiment ID: `certificate-s4-approval`
- Date: 2026-11-14
- Status: completed; six approval cases independently verified
- Scope: certificate for the S3 approval gate.

## Question

Can the approval gate be packaged as a self-contained, independently checkable
certificate and included in the bundle?

## Method

Emit schema, plan duration and seven gate cases, plus four time-varying
approval-schedule cases (single/multi x all-approved/reject-day20). The Python verifier recomputes expected
execution and blocker prefixes.

Prediction: `certificate=ok`.

## Limitation

Plan duration is emitted by Rust; the check does not recompute S1 closure.
