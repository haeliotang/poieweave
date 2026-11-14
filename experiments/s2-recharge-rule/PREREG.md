# S2 Recharge Rule — pre-registration

- Experiment ID: `s2-recharge-rule`
- Date: 2026-11-14
- Status: completed; switch tracks average recharge
- Scope: vary PV recharge output 10/20/40 kWh over 8 h.

## Question

If average recharge changes, does the first energy-limited load track it?

## Method

Set recharge output to 10/20/40 kWh over 8 h (1.25/2.5/5 kW average), sweep
load, record the first energy-limited value.

Prediction: threshold ≈ average recharge + ~0.1 kW.

## Limitation

One model/schedule; 0.1 kW resolution.
