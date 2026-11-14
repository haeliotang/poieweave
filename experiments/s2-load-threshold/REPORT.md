# S2 Load Threshold — report

- Date: 2026-11-14
- Status: regime switch located at 2.6 kW
- Runner: `cargo run --locked -q -p poie-viability --bin poie-load-threshold`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	survived_days	blocked_by
2.0	60	cyclic process dependency at turn-sleeve-p1
2.1	60	cyclic process dependency at turn-sleeve-p1
2.2	60	cyclic process dependency at turn-sleeve-p1
2.3	60	cyclic process dependency at turn-sleeve-p1
2.4	60	cyclic process dependency at turn-sleeve-p1
2.5	60	cyclic process dependency at turn-sleeve-p1
2.6	16	energy_depleted
2.7	8	energy_depleted
2.8	5	energy_depleted
2.9	4	energy_depleted
3.0	3	energy_depleted
```

## 2. Findings

- The switch is sharp: up to 2.5 kW the horizon stays 60 days (material path);
  at 2.6 kW it drops to 16 days with `energy_depleted`.
- Above the switch, each additional 0.1 kW reduces the horizon quickly.
- This gives a concrete, testable load boundary for the current model.

## 3. Limits

- One model and one failure schedule.
- 0.1 kW resolution.
