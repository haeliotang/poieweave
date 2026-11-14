# S4 Approval Gate — report

- Date: 2026-11-14
- Status: independent Python gate agrees on all six cases
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval-export | python3 experiments/s4-approval-gate/verify.py`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
cases=6 ok=6
OK
```

## 2. Findings

- The Python verifier independently decides, from `decision`, `lease_s`,
  `plan_duration_s`, `digest_ok` and `revoke_at_s`, whether a case executes and
  which blocker prefix it must have.
- All six exported cases agree, including the short-lease and wrong-digest
  fail-closed paths.
- The revoke case is checked as executed (initial repair) with an
  `approval_revoked` blocker.

## 3. Significance

- This extends S4 from receipts and plans to the approval gate itself.
- Shared contract: field names and reason prefixes; the decision logic is a
  separate implementation.

## 4. Limits

- `plan_duration_s` is exported by Rust, not recomputed by Python.
- Six cases; no multi-service approval schedule.
