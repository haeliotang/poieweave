# S3 Approval Revocation — pre-registration

- Experiment ID: `s3-revocation`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: revocation deadline in the single-service recovery loop.

## Question

Does a revoked approval stop further repairs, and is the block recorded?

## Method

Base-05 `potable-water`, day-10 pump failure, approved 8 h lease. Revoke at
day 5 and at t=0; run to 30 days.

Prediction: day-5 revocation blocks at the day-10 event; t=0 revocation blocks
immediately.

## Limitation

Isolated simulation; one service/event.
