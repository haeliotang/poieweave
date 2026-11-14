# S3 Approval Matrix — pre-registration

- Experiment ID: `s3-approval-matrix`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: approval decision × lease × digest × revocation on one failure.

## Question

Do all approval conditions fail closed, and does revocation interact correctly
with the initial repair?

## Method

Base-08, pump failure at day 10, 30-day horizon. Cases: required, rejected,
lease 1 h, approved 8 h, wrong digest, approved + revoke day 5.

Prediction: all non-approved cases fail closed; revocation at day 5 preserves
the t=0 repair but blocks the day-10 event.

## Limitation

One failure/policy set.
