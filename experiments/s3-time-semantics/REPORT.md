# S3 Time Semantics — report

- Date: 2026-11-14
- Status: dense failures expose timing; sparse horizons are unchanged
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-semantics`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
case	load_kw	mode	event_times_h	survived_days
dense	0.0	instant-repair	[0.0, 1.0]	1
dense	0.0	event-driven	[3.0, 6.0]	1
sparse	0.0	instant-repair	-	60
sparse	0.0	event-driven	-	60
sparse	2.6	instant-repair	-	16
sparse	2.6	event-driven	-	16
```

## 2. Findings

- Dense failures: the old loop processes at 0/1 h, the event-driven loop at
  3/6 h, because each 3 h repair advances the clock.
- Sparse 10-day failures: both semantics give 60 days at zero load and 16 days at
  2.6 kW; the repair overhead is too small to change the horizon.
- Time semantics therefore changes event order/timing first; horizon changes
  require repairs dense enough to consume a meaningful fraction of the horizon.

## 3. Significance

- The event-driven function is a drop-in alternative for later migration.
- It shows why the earlier boundary results remain stable for sparse schedules.

## 4. Limits

- One dense and one sparse schedule; no scheduler/preemption.
- Existing `simulate_*_with_load` functions are unchanged.
