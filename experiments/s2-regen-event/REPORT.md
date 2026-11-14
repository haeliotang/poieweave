# S2 Regeneration × Time Semantics — report

- Date: 2026-11-14
- Status: regeneration horizons are invariant at 3 h repairs
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-event`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
model	interval_days	instant_days	event_driven_days
base-05	10	40	40
base-05	15	60	60
base-05	20	80	80
base-05	30	90	90
base-08	10	60	60
base-08	15	90	90
base-08	20	90	90
base-08	30	90	90
```

## 2. Findings

- Every regeneration horizon is identical under both semantics.
- The nominal 3 h repair is negligible against 10-30 day failure intervals, so
  advancing the clock does not change the number of failures inside 90 days.

## 3. Significance

- This is a negative control for the regeneration result: the 40/60/80/90
  horizons are not artifacts of instantaneous repair semantics at nominal
  durations.
- It further localizes the time-semantics effect to long-repair scenarios.

## 4. Limits

- Nominal 3 h repair; no long-repair variant.
