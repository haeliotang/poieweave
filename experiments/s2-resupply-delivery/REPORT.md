# S2 External Resupply Deliveries — report

- Date: 2026-11-14
- Status: resupply extends survival from 30 to 40 days, but is not frequent enough
- Runner: `cargo run --locked -q -p poie-viability --bin poie-resupply-delivery`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	insert_stock	survived_days	blocked_by
one-insert-no-resupply	1	30	missing resource cutting-insert: need 1, have 0
four-inserts-no-resupply	4	60	cyclic process dependency at turn-sleeve-p1
one-insert-resupply	1	40	missing resource cutting-insert: need 1, have 0
```

## 2. Findings

- One insert without resupply: 30 days, then the insert is exhausted.
- Four inserts without resupply: 60 days, then the material/cycle limit binds.
- One insert plus deliveries at days 30/60: 40 days; the day-30 delivery buys
  one more repair, then the day-40 failure runs out before day 60.

## 3. Significance

- Resupply is not a binary fix: delivery frequency relative to repair cadence
  determines whether it closes the gap.
- In this schedule, a 30-day cadence is too slow for 10-day failures.

## 4. Limits

- One consumable, two deliveries; model-level units.
