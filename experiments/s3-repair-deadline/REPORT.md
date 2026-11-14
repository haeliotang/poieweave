# S3 Repair Deadline — report

- Date: 2026-11-14
- Status: clock-advanced repair misses a deadline the instant model meets
- Runner: `cargo run --locked -q -p poie-viability --bin poie-repair-deadline`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
repair_days	failure_day	deadline_day	instant_complete	instant_meets	clock_advanced_complete	clock_advanced_meets
3	10	11	10	true	13	false
```

## 2. Findings

- A 3-day repair starting at day 10 completes at day 13 under clock-advanced
  semantics, missing an 11-day deadline.
- The instantaneous model treats it as complete at the failure day and meets the
  deadline.
- This is the clearest case where time semantics changes a binary outcome rather
  than only a timestamp.

## 3. Significance

- Deadline feasibility must use repair completion time, not failure time.
- It gives a concrete migration target for the existing survival functions.

## 4. Limits

- Arithmetic completion model; one repair and one deadline.
- No scheduler or preemption.
