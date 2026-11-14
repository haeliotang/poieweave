# S3 Approval Matrix — report

- Date: 2026-11-14
- Status: all approval conditions fail closed; revocation interacts as expected
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval-matrix`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
case	executed	survived_days	blocked_by
required	false	0	approval_required:pump-p1
rejected	false	0	approval_rejected:pump-p1
lease-1h	false	0	approval_lease_expired:pump-p1: need 10800s, lease 3600s
approved-8h	true	30	none
wrong-digest	false	0	approval_digest_mismatch:pump-p1
revoked-day5	true	10	approval_revoked
```

## 2. Findings

- Required, rejected, short lease, and wrong digest all stop with no execution.
- Approved 8 h runs to the full 30-day horizon.
- Revocation at day 5 allows the initial t=0 repair but blocks the day-10 event,
  ending at day 10 with `approval_revoked`.

## 3. Significance

- The approval gate is fail-closed across decision, lease, and digest checks.
- Revocation is time-scoped: earlier applied plans remain, later proposals are
  rejected.

## 4. Limits

- One failure/policy set; model-level units.
