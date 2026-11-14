# S4 Energy Model — pre-registration

- Experiment ID: `s4-energy-model`
- Date: 2026-11-14
- Status: completed; independent linear model matches 4/4 horizons
- Scope: no-event energy horizons under four loads.

## Question

Can an independent model reproduce the pure energy-depletion horizons?

## Method

Rust exports no-event horizons for loads 2.5/2.6/3.0/4.0 kW. A Python model reads
the scenario TOML, derives initial electric energy and recharge inflow, and
computes the depletion time without calling Rust.

Prediction: all four integer days match.

## Limitation

No failures, no repair consumption, no material constraints; one scenario.
