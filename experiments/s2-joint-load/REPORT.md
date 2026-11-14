# S2 Joint Frontier Under Load — report

- Date: 2026-11-14
- Status: load regime overrides the bronze frontier
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-load`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	bronze_kg	survived_days	blocked_by
2	3.0	60	cyclic process dependency at turn-sleeve-p1
2	6.0	90	none
3	3.0	3	energy_depleted
3	6.0	3	energy_depleted
```

## 2. Findings

- At 2 kW the previous bronze frontier still holds: 3 kg -> 60 days,
  6 kg -> full 90 days.
- At 3 kW the energy regime dominates: both bronze stocks stop at 3 days with
  `energy_depleted`.
- Bronze stock no longer matters once sustained load exceeds average recharge.

## 3. Significance

- The binding frontier is regime-dependent: material/bronze below the load
  switch, energy above it.
- This connects the consumable frontier to the 2.6 kW recharge rule.

## 4. Limits

- Two loads; one policy.
- Model-level units.
