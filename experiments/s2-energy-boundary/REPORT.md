# S2 Energy Boundary — report

- Date: 2026-11-14
- Status: load threshold dominates; capacity/charge inert in tested range
- Runner: `cargo run --locked -q -p poie-viability --bin poie-energy-boundary`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	capacity_kwh	charge_kw	survived_days	blocked_by
2	100	3	60	cyclic process dependency at turn-sleeve-p1
...
3	300	30	3	energy_depleted
```

All 2 kW rows survive 60 days (material-limited); all 3 kW rows survive 3 days
(energy-limited), regardless of capacity or charge power.

## 2. Findings

- In the tested range, the load threshold (2 vs 3 kW) dominates.
- Capacity 100→300 kWh and charge 3→30 kW do not change the horizon.
- This is consistent with the short-horizon energy accounting: the site crosses
  the energy-limited regime at ~3 kW; within a regime, these parameters are not
  binding at the tested values.

## 3. Limits

- Three values per parameter; no continuous boundary search.
- Model-level units.
