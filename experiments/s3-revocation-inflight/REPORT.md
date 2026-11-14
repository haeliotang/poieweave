# S3 In-Flight Revocation — report

- Date: 2026-11-14
- Status: in-flight revocation cancels before applying the plan
- Runner: `cargo run --locked -q -p poie-viability --bin poie-revocation-boundary`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
revoke_at_s	executed	survived_s	blocked_by	receipts
0	false	0	approval_revoked	0
1	false	0	approval_revoked_mid_plan:pump-p1: revoke 1s during 10800s plan	0
10799	false	0	approval_revoked_mid_plan:pump-p1: revoke 10799s during 10800s plan	0
10800	true	86400	none	1
```

## 2. Findings

- A revocation at t=0 is rejected before execution.
- A revocation strictly inside the 3 h plan now cancels the plan before any
  effect: no execution and no receipt.
- A revocation exactly at the plan end (10800 s) does not cancel; the plan
  completes and produces a receipt.

## 3. Fix

`recover_service` now checks `revoke_at_s` against `[now, now + plan_duration)`
after approval and before applying the plan. A mid-plan revocation returns
`approval_revoked_mid_plan:<asset>`.

## 4. Significance

- This closes the earlier gap where revocation only affected future proposals.
- The boundary is half-open: revoke at completion time is allowed; strictly
  before completion cancels.

## 5. Limits

- Cancellation is atomic before any plan effect; no partial execution rollback.
- One plan/policy; model-level units.
