# Multi-Service Recovery Certificate — pre-registration

- Experiment ID: `certificate-s3-multi`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: machine-checkable multi-service S3 recovery trace.

## Question

Can a two-service, two-event recovery trace be packaged with model identity and
receipt chain for independent verification?

## Method

Certificate binds Base-05 SHA-256, services, two events, approved 8 h lease,
three receipts and an isolated-simulation claim scope. Python checks the hash,
services, events, receipts, inventory chain and scope.

Prediction: `certificate=ok`; bundle includes `S3-multi`.

## Limitation

Self-issued; isolated simulation; two services/events.
