# S2 Redundancy Under Load — report

- Date: 2026-11-14
- Status: redundancy only helps below the energy switch
- Runner: `cargo run --locked -q -p poie-viability --bin poie-redundancy-load`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	load_kw	survived_days	blocked_by
baseline	2.5	60	cyclic process dependency at turn-sleeve-p1
baseline	2.6	16	energy_depleted
baseline	3.0	3	energy_depleted
redundant-pump	2.5	90	none
redundant-pump	2.6	17	energy_depleted
redundant-pump	3.0	3	energy_depleted
```

## 2. Findings

- At 2.5 kW redundancy removes the material/cycle blocker: 60 -> 90 days.
- At 2.6 kW redundancy adds only 1 day (16 -> 17); energy still depletes.
- At 3.0 kW redundancy changes nothing (3 days, energy_depleted).

## 3. Significance

- Asset redundancy is not a substitute for energy margin on the wrong side of
  the recharge switch.
- It shows two different interventions for two different regimes:
  restorers/redundancy for material limits, energy capacity/recharge for load
  limits.

## 4. Limits

- Two modes, three loads; model-level units.
