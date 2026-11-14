# S2 Regeneration Cost under Load — report

- Date: 2026-11-14
- Status: negative interaction result
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-cost-load`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
duration_x	load_kw	survived_days	blocked_by
1.0	0.0	60	cyclic process dependency at turn-sleeve-p1
1.0	2.0	60	cyclic process dependency at turn-sleeve-p1
1.0	3.0	3	energy_depleted
4.0	0.0	60	cyclic process dependency at turn-sleeve-p1
4.0	2.0	60	cyclic process dependency at turn-sleeve-p1
4.0	3.0	3	energy_depleted
```

## 2. Findings

- At 3 kW the site becomes energy-limited (3 days), but the 4× recycling
  duration still does not change the horizon.
- Recycling duration and load are not interacting in the current model.
- This strengthens the earlier negative result: cost is not on the critical
  path yet.

## 3. Limits

- Only recycle duration; no mass/energy-per-kg model.
- One model family and horizon.
