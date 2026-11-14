# S3 Lease Boundary — pre-registration

- Experiment ID: `s3-lease-boundary`
- Date: 2026-11-14
- Status: completed; exact-fit lease is accepted
- Scope: 3 h pump repair with lease 10799/10800/10801 s.

## Question

Is the lease check inclusive at the exact plan duration?

## Method

Approved policy, empty failure list, horizon 1 day. Lease one second below,
equal to, and one second above the 10800 s plan.

Prediction: below fails, equal and above execute.

## Limitation

One plan and policy.
