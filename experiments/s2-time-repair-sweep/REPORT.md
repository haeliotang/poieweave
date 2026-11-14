# S2 Repair-Duration Sweep — report

- Date: 2026-11-15
- Status: effect starts between 1 and 2 day repairs; monotone thereafter
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-repair-sweep`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
repair_days	instant_days	event_driven_days
0.125	6	6
0.5	6	6
1	6	6
2	6	7
3	6	10
5	6	16
```

## 2. Findings

- For repairs up to 1 day, both semantics give 6 days.
- At 2 days the event-driven horizon rises to 7, then 10 at 3 days and 16 at
  5 days.
- The instantaneous horizon stays at 6 throughout: the resource limit is
  consumed by the same number of repairs.

## 3. Significance

- This localizes the time-semantics threshold at roughly half the failure
  interval in this scenario.
- It gives a quantitative answer to "when must repair duration be modeled?".

## 4. Limits

- One failure cadence; one resource-limited scenario; model-level units.
