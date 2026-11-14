# S3 Lease Renewal — report

- Date: 2026-11-14
- Status: lease is checked per proposal, so renewal is implicit
- Runner: `cargo run --locked -q -p poie-viability --bin poie-lease-renewal`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
events=3 lease_h=8 receipts=4 survived_days=90 blocked_by=none
```

## 2. Findings

- Three failures at days 10/20/30 plus the initial repair all execute with a
  single 8 h lease policy.
- Receipts = 4, one per applied plan.
- `check_approval` is called per plan against the policy lease, so the lease is
  effectively renewed for each proposal.

## 3. Significance

- There is no absolute-lease expiry across the horizon; the lease bounds one
  plan, not the whole run.
- An explicit renewal loop can be modelled by issuing a fresh policy per
  proposal, but the current single-policy API already has per-plan semantics.

## 4. Limits

- No explicit renewal/denial event between proposals.
- Model-level units.
