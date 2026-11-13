# S3 Approval State and Lease — pre-registration

- Experiment ID: `s3-approval-lease`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: isolated simulation; not an operator workflow or hardware authority.

## Question

Do explicit approval states and a bounded approval lease change recovery
behavior compared with a boolean approval flag?

## Method

Base-05, `potable-water`, 30-day horizon, pump failures at days 10 and 20.
Run the recovery loop with:

- Approved, 8 h lease;
- Required (no decision), 4 h lease;
- Rejected, 4 h lease;
- Approved, 2 h lease.

A plan is applied only if its duration is within the lease; otherwise the loop
records `approval_lease_expired` and stops without applying it.

## Prediction

- Approved 8 h survives 30 days.
- Required and Rejected stop at t=0.
- Approved 2 h expires because the initial pump repair needs 3 h.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

The lease is a study parameter; there is no signed approval, operator identity,
renewal, or execution gate yet.
