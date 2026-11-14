# S2 Joint Frontier at the Load Switch — report

- Date: 2026-11-14
- Status: sharp switch between 2.5 and 2.6 kW
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-boundary`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	bronze_kg	survived_days	blocked_by
2.4	3.0	60	cyclic process dependency at turn-sleeve-p1
2.4	6.0	90	none
2.5	3.0	60	cyclic process dependency at turn-sleeve-p1
2.5	6.0	90	none
2.6	3.0	16	energy_depleted
2.6	6.0	16	energy_depleted
```

## 2. Findings

- At 2.4 and 2.5 kW the bronze frontier is unchanged: 3 kg -> 60 days,
  6 kg -> full 90 days.
- At 2.6 kW the energy regime takes over: both bronze stocks stop at 16 days
  with `energy_depleted`.
- The switch is sharp, matching the recharge bound (2.5 kW average).

## 3. Significance

- This locates the joint frontier transition exactly: below/at average recharge
  the consumable binds; above it energy binds.
- It rules out a gradual blend between the two regimes in this model.

## 4. Limits

- Three load points; one policy.
- Model-level units.
