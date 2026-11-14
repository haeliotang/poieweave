# S4 Approval Gate — pre-registration

- Experiment ID: `s4-approval-gate`
- Date: 2026-11-14
- Status: completed; independent Python gate check agrees on 6/6 cases
- Scope: independent verification of S3 approval semantics.

## Question

Can a separate implementation reproduce the approval gate's decisions from the
exported policy fields?

## Method

Export six cases (required, rejected, short lease, approved, wrong digest,
revoked). The Python verifier recomputes expected execution and blocking reason
from decision, lease, plan duration, digest flag and revocation.

Prediction: all six cases agree.

## Limitation

Plan duration is exported by the Rust side; the Python check does not recompute
S1 closure.
