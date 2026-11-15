# S5 Scheduled Survival — report

- Date: 2026-11-15
- Status: two-robot batch completes 8 h vs 6 h
- Runner: `cargo run --locked -q -p poie-viability --bin poie-scheduled-survival`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
robots	completion_h	horizon_days	blocked
1	8	30	none
2	6	30	none
```

## 2. Findings

- Initial pump repair takes 3 h on the shared clock.
- Pump and lathe failures at t=0 are batched and scheduled together.
- One robot takes 5 h for the batch (3 h + 2 h), completing at 8 h.
- Two robots overlap them, taking 3 h, completing at 6 h.
- The survival loop therefore consumes scheduler makespan, not a sum of plans.

## 3. Binding and order

- The exact `Plan` objects scheduled are the ones applied by `apply_plans`; the
  loop no longer re-plans through `restore_services` after scheduling.
- Energy is settled over the repair interval **before** asset status is updated,
  so a failed PV asset cannot generate during its own repair.
- Inventory deltas from multiple plans are summed against the planning snapshot,
  then added to the post-energy balance; a test covers this exact ordering.
- Status changes are committed only after inventory validation; a negative test
  confirms no partial asset status.

## 4. Significance

- First survival path that uses `schedule_plans` and `EventClock`.
- It makes resource occupancy affect horizon timing, a prerequisite for S5.

## 5. Initial repair and termination

- Initial repair uses the same transaction: planning snapshot, energy settlement,
  inventory delta, validation, commit.
- `SurvivalTermination` distinguishes `CompletedHorizon`, `Blocked` and
  `IncompleteAtHorizon`; a repair still in flight at the horizon is not reported
  as completed.

## 6. Limits

- Coarse inventory coupling per batch; no preemption; model-level units.
