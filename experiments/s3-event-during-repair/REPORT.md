# S3 Event During Repair — report

- Date: 2026-11-14
- Status: the model has no mid-repair interruption
- Runner: `cargo run --locked -q -p poie-viability --bin poie-event-during-repair`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
repair_s	event_s	inside_repair	receipts	blocked_by
10800	3600	true	1	no restorer for filter-1
```

## 2. Findings

- The filter event occurs at 1 h, inside the 3 h pump repair window.
- The simulation still has one receipt (the pump repair) and processes the
  filter event at its absolute time.
- There is no "repair interrupted" state or marker; plans are applied atomically
  and durations do not advance the simulation clock.

## 3. Significance

- This is a concrete S3 limitation: repair duration is checked against horizon
  and lease but does not defer or interrupt later events.
- It explains why in-flight cancellation needed an explicit revocation check, and
  it sets a target for a future event-driven scheduler.

## 4. Limits

- Gap detection only; interruption/preemption is not implemented.
- Model-level units.
