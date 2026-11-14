# S3 Lease Boundary — report

- Date: 2026-11-14
- Status: exact lease equality is accepted
- Runner: `cargo run --locked -q -p poie-viability --bin poie-lease-boundary`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
lease_s	executed	blocked_by	receipts
10799	false	approval_lease_expired:pump-p1: need 10800s, lease 10799s	0
10800	true	none	1
10801	true	none	1
```

## 2. Findings

- A lease 1 second shorter than the 3 h plan expires without execution.
- A lease exactly equal to the plan duration is accepted and produces a receipt.
- The comparison is `duration > lease` (strict), not `>=`.

## 3. Significance

- Pins down an approval boundary that matters for operator scheduling: an exact
  lease is sufficient.
- It adds a precise negative/positive pair to the S3 lease evidence.

## 4. Limits

- One plan and policy; no renewal/revocation interaction.
