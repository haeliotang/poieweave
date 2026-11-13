# Recovery Certificate — pre-registration

- Experiment ID: `certificate-s3-recovery`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: machine-checkable isolated S3 evidence.

## Question

Can an S3 approval-gated recovery trace be packaged as a certificate that a
separate implementation can verify?

## Method

Generate a certificate containing:

- Base-05 model path + SHA-256;
- service, horizon, approval decision and lease;
- failure event;
- both recovery receipts;
- explicit isolated-simulation claim scope.

Python verifies the model hash, service/approval/events, receipt digests and
inventory chain, then checks the scope.

Prediction: `certificate=ok`.

## Limitation

Self-issued, unsigned; isolated simulation only.
