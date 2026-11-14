# S2 Capacity Semantics — pre-registration

- Experiment ID: `s2-regen-capacity-semantics`
- Date: 2026-11-14
- Status: completed; capacity now rejects oversized initial inventory
- Scope: initial energy 500 kWh, capacity 200–600 kWh.

## Question

Does `BatteryLimits.capacity_kwh` cap the initial electric-energy stock?

## Method

Forced recycling path, initial energy fixed at 500 kWh, capacities 200/400/500/600
kWh; find first blocked recycle-energy multiplier.

Prediction: if capacity caps initial SOC, threshold should be near capacity plus
margin; otherwise it stays near 500.

## Limitation

Forced path; one model family.
