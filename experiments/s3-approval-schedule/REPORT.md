# S3 Approval Schedule — report

- Date: 2026-11-14
- Status: a time-varying approval schedule rejects the day-20 proposal
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval-schedule`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
case	receipts	survived_days	blocked_by
approved-all	3	30	none
reject-day20	2	20	approval_rejected:pump-p1
```

## 2. Findings

- All-approved: initial repair plus day-10 and day-20 repairs, three receipts,
  full 30 days.
- Schedule with a day-20 rejection: initial and day-10 repairs succeed (two
  receipts), then the day-20 proposal is rejected and the run stops at day 20.

## 3. Significance

- Approval can now vary over time instead of being a single policy for the whole
  horizon.
- This closes part of the explicit renewal/denial gap.

## 4. Limits

- One service; rejection stops the run; model-level units.
