# S2 Threshold vs Battery Capacity — pre-registration

- Experiment ID: `s2-regen-threshold-battery`
- Date: 2026-11-14
- Status: completed; threshold invariant to capacity
- Scope: forced recycling path, battery capacities 100–400 kWh.

## Question

Does the 60/61 recycle-energy threshold scale with battery capacity?

## Method

Same forced path as `s2-regen-threshold`; capacities 100/200/300/400 kWh; find
max safe and first blocked multiplier.

Prediction: larger capacity should raise the threshold if capacity seeds the
available energy.

## Limitation

One model family; forced path.
