# S3 Approval State and Lease — report

- Date: 2026-11-14
- Status: isolated model-level simulation; not a hardware authority claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
decision	lease_h	survived_days	blocked_by
Approved	8.0	30	none
Required	4.0	0	approval_required:pump-p1
Rejected	4.0	0	approval_rejected:pump-p1
Approved	2.0	0	approval_lease_expired:pump-p1: need 10800s, lease 7200s
```

## 2. Findings

- Approval is now a three-state decision: required, approved, rejected.
- A plan is executed only when approved **and** its duration fits the lease.
- An approved but too-short lease (2 h vs 3 h repair) expires; no repair is
  applied.
- A valid 8 h lease repairs the initial pump and both failures and reaches the
  30-day horizon.
- This separates authorization from capability: the S1 plan can be valid while
  the authorization window is not.

## 3. Limits

- No operator identity/signature, renewal, revocation, or execution receipt.
- The lease is checked against the modelled repair duration only.
- No hardware dispatch; no physical safety implication.

## 4. Next

1. Add proposal lifecycle transitions and renewal/revocation.
2. Bind approval to a plan digest and evidence snapshot.
3. Connect the approved plan to the existing simulated execution gate.
4. Add independent post-execution verification.
