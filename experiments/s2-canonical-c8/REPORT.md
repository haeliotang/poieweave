# S2 Canonical C8 Migration — report

- Date: 2026-11-16
- Status: EXPECTED_AGREEMENT on 11/11 rows
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c8`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load	legacy	canonical	energy	termination
2.0	60	60	200.0	cycle
2.1	60	60	200.0	cycle
2.2	60	60	200.0	cycle
2.3	60	60	200.0	cycle
2.4	60	60	170.3	cycle
2.5	60	60	26.3	cycle
2.6	16	16	0.0	energy_depleted
2.7	8	8	0.0	energy_depleted
2.8	5	5	0.0	energy_depleted
2.9	4	4	0.0	energy_depleted
3.0	3	3	0.0	energy_depleted
```

## 2. Findings

- Regime switch remains at 2.6 kW.
- 2.0–2.5 kW: material-limited at 60 days.
- 2.6–3.0 kW: energy-limited at 16/8/5/4/3 days.
- Final energy matches on every row, including partial 170.3/26.3 kWh cases.
- Classification: `EXPECTED_AGREEMENT`.

## 3. Limits

- Constant load; model-level units.
