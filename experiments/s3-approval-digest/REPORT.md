# S3 Approval Digest — report

- Date: 2026-11-14
- Status: isolated model-level simulation; not a hardware authority claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval-digest`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
correct_plan_digest=7e4742b19d617d5cb7b96b4c4dcef25c26a550a8a3365fd72a9bd1bb704b427a
correct	survived_days=30	blocked_by=none
wrong	survived_days=0	blocked_by=approval_digest_mismatch:pump-p1
changed_plan	survived_days=10	blocked_by=approval_digest_mismatch:pump-p1
```

## 2. Findings

- The approval is bound to a SHA-256 digest of the actual S1 plan, not just to an
  asset name.
- A wrong digest is rejected before any repair.
- If a new failure changes the required plan, the old approval no longer
  matches; the loop stops and requires a new approval.
- This is closer to "approve this exact evidence", not "approve repairs in
  general".

## 3. Limits

- The digest domain is internal; there is no signed authorization or operator
  identity.
- No plan versioning/expiry beyond the lease.
- No hardware dispatch or physical safety claim.

## 4. Next

1. Include model hash, inventory ledger and evidence snapshot in the digest.
2. Add proposal/approval versions and re-approval workflow.
3. Emit an execution receipt bound to the same digest.
