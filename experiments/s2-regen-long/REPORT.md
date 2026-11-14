# S2 Regeneration × Long Repairs — report

- Date: 2026-11-14
- Status: dense failures plus 5-day repairs change the regeneration horizon
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-long`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
model	interval_days	instant_days	event_driven_days
base-05	2	8	20
base-05	3	12	20
base-05	5	20	20
base-05	10	40	40
base-08	2	12	31
base-08	3	18	31
base-08	5	30	31
base-08	10	60	60
```

## 2. Findings

- At 2/3/5-day failures, the event-driven loop survives longer: Base-05 8/12/20
  vs 20/20/20; Base-08 12/18/30 vs 31/31/31.
- At 10-day intervals the horizons coincide again (40/60).
- The transition is driven by repair duration (5 days) relative to the failure
  interval, not by regeneration per se.

## 3. Significance

- Long repairs can change regeneration-related horizons in the dense-failure
  regime, complementing the nominal-repair negative control.
- It defines the regime where time semantics must be modeled for regeneration.

## 4. Limits

- One long-repair setting; model-level units.
