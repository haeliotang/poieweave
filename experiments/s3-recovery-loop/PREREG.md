# S3 Recovery Loop — pre-registration

- Experiment ID: `s3-recovery-loop`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: isolated simulation; no hardware authority.

## Question

Can the recovery loop be gated by explicit approval, apply only approved S1
plans, and verify the service after each repair?

## Method

Base-05, `potable-water`, 30-day horizon, pump failures at day 10 and 20. Run
the loop once with `approved=false` and once with `approved=true`. The loop uses
the shared inventory, applies the S1 closure plan, and re-checks the service
requirements after every repair.

## Prediction

- No approval: stop at t=0 with `approval_required:pump-p1`, because the initial
  pump is degraded.
- Approval: repair at t=0 and after each event; survive the full 30 days with
  verified=true entries and no blocked entry.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

Approval is a boolean study parameter, not a real operator or execution gate;
no leases, expiry, hardware dispatch or evidence certificate yet.
