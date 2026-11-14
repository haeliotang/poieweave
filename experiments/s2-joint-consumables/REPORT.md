# S2 Joint Consumables — report

- Date: 2026-11-14
- Status: sharpening shifts the bottleneck to bronze
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-consumables`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
inserts	bronze_kg	survived_days	blocked_by
2	0.0	30	cyclic process dependency at turn-sleeve-p1
2	1.5	40	cyclic process dependency at turn-sleeve-p1
2	3.0	60	cyclic process dependency at turn-sleeve-p1
2	6.0	90	none
4	0.0	30	cyclic process dependency at turn-sleeve-p1
4	1.5	40	cyclic process dependency at turn-sleeve-p1
4	3.0	60	cyclic process dependency at turn-sleeve-p1
4	6.0	90	none
```

## 2. Findings

- With sharpening enabled, insert count (2 vs 4) no longer changes the horizon.
- Bronze stock drives the horizon: 0/1.5/3/6 kg -> 30/40/60/90+ days.
- At 6 kg bronze the run reaches the full 90-day horizon with no blocker.
- The blocker remains the cyclic `turn-sleeve-p1` dependency, but its arrival
  time is set by bronze availability.

## 3. Significance

- Regenerating inserts shifts the binding constraint to bronze.
- This is a concrete joint-consumable frontier: solving one wear part exposes
  the next.
- It also shows a case where the system reaches the full horizon (90 days)
  rather than stopping early.

## 4. Limits

- Two consumables; one policy.
- Model-level units.
