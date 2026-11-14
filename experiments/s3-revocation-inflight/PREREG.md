# S3 In-Flight Revocation — pre-registration

- Experiment ID: `s3-revocation-inflight`
- Date: 2026-11-14
- Status: completed; revocation during a plan does not cancel it
- Scope: revoke at 0/1/10799/10800 s for a 3 h plan.

## Question

Does revocation during an in-flight plan stop or invalidate execution?

## Method

Approved 3 h repair at t=0; revoke at 0, 1, 10799, 10800 s; record execution
and receipts.

Prediction: only a revocation at t=0 prevents the plan.

## Limitation

One plan/policy; no mid-plan cancellation model.
