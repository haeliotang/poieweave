# S4 Energy Model — report

- Date: 2026-11-14
- Status: independent linear model matches 4/4 horizons
- Runner: `cargo run --locked -q -p poie-viability --bin poie-energy-horizon > /tmp/eh.tsv`
  then `python3 experiments/s4-energy-model/verify.py /tmp/eh.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=4 matched=4
```

## 2. Findings

- Python reads `scenarios/base-08/model.toml`, sums the electric-energy lots
  (40 kWh) and derives recharge inflow (20 kWh / 8 h = 2.5 kW).
- It computes `initial / (load - inflow)` for energy-limited loads and the
  horizon for stable loads, then compares integer days with Rust.
- All four loads agree: 90, 16, 3, 1 days.

## 3. Significance

- This is the first independent process/energy model in S4: it does not call the
  Rust simulator and shares only the scenario file and unit contract.
- It is deliberately narrow: no events, repairs, or material limits.

## 4. Limits

- One scenario, no events or repair consumption.
- The cross-check is arithmetic, not a full process model or FMI.
