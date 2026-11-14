# S2 Resupply × Time Semantics — report

- Date: 2026-11-14
- Status: resupply cadence dominates; both semantics give the same horizons
- Runner: `cargo run --locked -q -p poie-viability --bin poie-resupply-time`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
interval_days	instant_days	event_driven_days	blocked_by
10	60	60	cyclic process dependency at turn-sleeve-p1
20	50	50	missing resource cutting-insert: need 1, have 0
30	40	40	missing resource cutting-insert: need 1, have 0
60	30	30	missing resource cutting-insert: need 1, have 0
```

## 2. Findings

- The horizons are identical under both semantics at every delivery cadence.
- The insert delivery schedule, not repair duration, decides how many repairs are
  possible; advancing the clock shifts event times but not the count before the
  insert limit is hit in this schedule.
- A negative composition result: adding resupply removes the time-semantics
  effect observed without resupply.

## 3. Significance

- The time-semantics effect depends on the binding constraint: it appeared in
  no-resupply consumable scenarios and disappears when resupply keeps pace.
- It prevents over-generalizing "clock advancement changes horizons".

## 4. Limits

- Insert-only; event-driven resupply variant omits energy/load.
- Model-level units.
