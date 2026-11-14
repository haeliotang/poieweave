# S3 In-Flight Revocation — report

- Date: 2026-11-14
- Status: only t=0 revocation prevents execution
- Runner: `cargo run --locked -q -p poie-viability --bin poie-revocation-boundary`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
revoke_at_s	executed	survived_s	blocked_by	receipts
0	false	0	approval_revoked	0
1	true	86400	none	1
10799	true	86400	none	1
10800	true	86400	none	1
```

## 2. Findings

- Revoking at t=0 prevents the plan: no execution and no receipt.
- Revoking at t=1 s or later does not stop the plan that started at t=0; the run
  still executes and produces a receipt.
- The simulation applies the plan atomically at the decision point and checks
  revocation only at loop boundaries, so an in-flight plan is not cancelled.

## 3. Significance

- This distinguishes two revocation semantics: "blocks future proposals" vs
  "cancels in-flight execution". The current model implements the former.
- It is a concrete gap for S3: cancellation during execution is not modeled.

## 4. Limits

- One plan/policy; no mid-plan cancellation model.
- Model-level units.
