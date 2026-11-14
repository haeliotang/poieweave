# S4 Approval-Schedule Model — report

- Date: 2026-11-14
- Status: independent receipts/horizon model matches 4/4
- Runner: `cargo run --locked -q -p poie-viability --bin poie-approval-schedule > /tmp/as.tsv`
  then `python3 experiments/s4-approval-schedule-model/verify.py /tmp/as.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=4 matched=4
```

## 2. Findings

- All-approved: initial + two events = 3 receipts, 30 days.
- Reject-day20: initial + day-10 event = 2 receipts, 20 days, rejection blocker.
- Multi-service rows share the counts; only the blocker gains a service prefix.

## 3. Significance

- The approval-schedule decision arithmetic now has an independent implementation.
- It complements the certificate and the Rust regression tests.

## 4. Limits

- Fixed schedule/events; no lease or digest logic.
