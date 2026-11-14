# S3 Approval Revocation — report

- Date: 2026-11-14
- Status: revocation deadline in the isolated recovery loop
- Runner: `cargo run --locked -q -p poie-viability --bin poie-revocation`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
revoke-day5: survived_days=10.0 blocked=approval_revoked
revoke-start: survived_days=0.0 blocked=approval_revoked
```

## 2. Findings

- Revocation after approval stops the loop at the next event; no repair is
  applied and the run ends at day 10.
- Revocation at t=0 blocks before the initial recovery plan.
- The block reason is explicit (`approval_revoked`), not a silent success.

## 3. Limits

- Isolated simulation, not a hardware authority path.
- One service and one event.
