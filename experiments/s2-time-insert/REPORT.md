# S2 Time-Semantics Insert Boundary — report

- Date: 2026-11-14
- Status: the insert boundary shifts from 6/8/10/12/12 to 15/20/26/31/31
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-insert`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
inserts	instant_days	event_driven_days
1	6	15
2	8	20
3	10	26
4	12	31
6	12	31
```

## 2. Findings

- With 5-day repairs, the clock-advanced horizon is much longer for every insert
  stock level.
- Both curves saturate: 12 days at 4 inserts instantaneously, 31 days at 4
  inserts with clock advance.
- The gap is caused by later failures being pushed past the horizon, not by
  different per-repair consumption.

## 3. Significance

- This migrates a previously measured insert-stock boundary to the new time
  semantics and shows the boundary is not semantics-invariant.
- It complements the grease boundary shift with a second consumable.

## 4. Limits

- Modified repair duration; one consumable; no scheduler.
