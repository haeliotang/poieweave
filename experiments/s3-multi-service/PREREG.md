# S3 Multi-Service Recovery — pre-registration

- Experiment ID: `s3-multi-service`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one approval-gated recovery loop across two services.

## Question

Can a single shared-inventory recovery loop keep more than one service verified?

## Method

Base-05, services `potable-water` and `workshop-operability`, day-10 pump
failure, 30-day horizon. Run with an approved 8 h lease and a 1 h lease.

Prediction: 8 h survives 30 days with receipts; 1 h lease blocks.

## Limitation

Isolated simulation; two services; one event.
