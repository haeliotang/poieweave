# S4 PV Failure Model — pre-registration

- Experiment ID: `s4-energy-pv`
- Date: 2026-11-14
- Status: completed; independent two-phase model matches 2/2
- Scope: PV failure at day 5, loads 2.0/2.5 kW.

## Question

Can an independent model reproduce the horizon after a PV failure, including
pre-failure charging and capacity?

## Method

Rust exports horizons for a pv-1 failure at day 5. Python reads the TOML (40 kWh
initial, 2.5 kW recharge, 200 kWh cap), subtracts the 0.5 kWh initial repair,
charges with net inflow for 5 days, then depletes at the load.

Prediction: both integer/rounded days match.

## Limitation

Two loads, one failure; no material constraints.
