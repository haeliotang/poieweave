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

## 3. Significance

- First survival path that uses `schedule_plans` and `EventClock`.
- It makes resource occupancy affect horizon timing, a prerequisite for S5.

## 4. Limits

- Coarse inventory coupling per batch; no preemption; model-level units.
