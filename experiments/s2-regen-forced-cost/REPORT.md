# S2 Forced Regeneration Cost — report

- Date: 2026-11-14
- Status: cost binds only when recycling is forced onto the critical path
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-forced-cost`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
energy_x	survived_days	blocked_by
1	10	cyclic process dependency at turn-sleeve-p1
10	10	cyclic process dependency at turn-sleeve-p1
50	10	cyclic process dependency at turn-sleeve-p1
100	0	missing resource electric-energy: need 360000000, have 216000000
```

## 2. Findings

- With no accepted sleeves and no bronze, the closure plan includes
  `recycle-bronze` before turning and replacing the sleeve.
- At 100× electricity, the plan is immediately blocked by missing energy
  (360 MJ needed, 216 MJ available).
- At 10×/50× it still survives 10 days (exact threshold is 60×/61×), so the current model has a very wide
  cost-insensitive band before energy binding.

## 3. Interpretation

This resolves the earlier apparent contradiction:

- when recycling is not scheduled, its cost is inert;
- when forced onto the path, the cost can bind;
- the threshold is high because a recycle run costs 1 kWh against a 200 kWh
  battery.

## 4. Limits

- Model-level units and one cost dimension.
- No mass/contamination/energy-per-kg model.
