# S3 Time Semantics — report

- Date: 2026-11-14
- Status: event-driven repair advances the clock; instantaneous does not
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-semantics`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	event_times_h	survived_days
instant-repair	[0.0, 1.0]	1
event-driven	[3.0, 6.0]	1
```

## 2. Findings

- The original loop processes the two failures at 0 h and 1 h; the applied 3 h
  repair does not move the clock.
- The new event-driven loop advances by 3 h per repair: the first event is
  deferred to 3 h and the second to 6 h.
- Both survive the 1-day horizon in this scenario; the difference is timing, not
  this horizon.

## 3. Significance

- This closes the `s3-event-during-repair` gap in a separate function without
  changing existing horizon results.
- It provides a concrete target for migrating survival runs to event-driven
  semantics.

## 4. Limits

- One scenario; no scheduler/preemption.
- Existing `simulate_*_with_load` functions are unchanged.
