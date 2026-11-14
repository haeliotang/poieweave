# S2 Resupply Frequency — report

- Date: 2026-11-14
- Status: cadence 10/20/30/60 days gives a monotone 60/50/40/30-day horizon
- Runner: `cargo run --locked -q -p poie-viability --bin poie-resupply-frequency`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
interval_days	survived_days	blocked_by
10	60	cyclic process dependency at turn-sleeve-p1
20	50	missing resource cutting-insert: need 1, have 0
30	40	missing resource cutting-insert: need 1, have 0
60	30	missing resource cutting-insert: need 1, have 0
```

## 2. Findings

- The horizon increases monotonically as the delivery interval shortens:
  60/50/40/30 days for 10/20/30/60-day cadences.
- At a 10-day cadence the insert is no longer the binder; the run stops at the
  material/cycle limit at 60 days.
- At 20/30/60 days the insert still binds.

## 3. Significance

- The delivery cadence is a first-class survival parameter, not a binary
  "resupplied or not" condition.
- A cadence at least as fast as the failure/repair cadence is needed to remove
  the insert limit in this scenario.

## 4. Limits

- One consumable, one failure cadence; model-level units.
