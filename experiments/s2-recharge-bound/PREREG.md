# S2 Recharge Bound — pre-registration

- Experiment ID: `s2-recharge-bound`
- Date: 2026-11-14
- Status: completed; threshold matches average recharge power
- Scope: explain the 2.6 kW switch.

## Question

Does the 2.6 kW material-to-energy switch correspond to the model's average PV
recharge power?

## Method

Read `recharge-energy` (20 kWh over 8 h = 2.5 kW), sweep load to find the first
energy-limited value, and compare.

Prediction: threshold just above average recharge power.

## Limitation

One model/schedule; 0.1 kW resolution.
