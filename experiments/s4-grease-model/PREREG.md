# S4 Grease Model — pre-registration

- Experiment ID: `s4-grease-model`
- Date: 2026-11-14
- Status: completed; independent repair-count model matches 6/6
- Scope: limited grease stock × repair cost.

## Question

Can an independent model reproduce how many repairs a limited grease stock
supports?

## Method

Python reads the Base-08 TOML for the grease stock and the cheap restorer cost,
uses the known 0.20 kg expensive cost, and simulates the t=0 repair plus 10-day
events. Compare with the Rust `poie-grease-cost` table.

Prediction: all six horizon values match.

## Limitation

One consumable; no other material constraints.
