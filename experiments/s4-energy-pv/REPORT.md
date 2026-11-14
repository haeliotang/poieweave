# S4 PV Failure Model — report

- Date: 2026-11-14
- Status: independent two-phase model matches 2/2 horizons
- Runner: `cargo run --locked -q -p poie-viability --bin poie-energy-pv > /tmp/epv.tsv`
  then `python3 experiments/s4-energy-pv/verify.py /tmp/epv.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=2 matched=2
```

Rust horizons: 7 days at 2.0 kW, 6 days at 2.5 kW.

## 2. Findings

- The initial t=0 repair consumes 0.5 kWh, leaving 39.5 kWh.
- Before the failure, load 2.0 kW has net +0.5 kW and charges to 99.5 kWh
  (below the 200 kWh cap); load 2.5 kW stays at 39.5 kWh.
- After the failure, depletion is `energy / load`, rounded to whole days.
- Python reproduces both horizons without calling the Rust simulator.

## 3. Significance

- This extends the independent model from a static no-event horizon to a
  two-phase fault scenario with charging, a capacity cap, and repair consumption.
- Shared contract: scenario TOML, units, 0.5 kWh initial repair, rounding.

## 4. Limits

- Two loads, one event; no material or repair-after-failure constraints.
