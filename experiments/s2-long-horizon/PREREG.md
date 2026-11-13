# S2 Long-Horizon Survival — pre-registration

- Experiment ID: `s2-long-horizon`
- Date: 2026-11-14
- Status: completed; final table in [REPORT.md](REPORT.md)
- Scope: deterministic greedy policy over Base-05; model-level only.

## Question

With external supply cut off, repeated pump failures and PV recharge on Base-05,
how long can `potable-water` be maintained using only local inventory,
production and the S1 repair closure?

## Policy

- Initial degraded pump is repaired before operation.
- Pump failures are scheduled every 10 days from day 10 to day 90.
- At each event, S1 closure plans the repair against the **current shared
  inventory**; the plan's remaining inventory becomes the next state.
- Between events, `recharge-energy` adds PV energy if `pv-1` is available.
- A blocked repair ends the run at that event; no intervention is allowed.

This is a greedy fixed policy, not an optimal long-horizon planner.

## Prediction

Base-05 has two accepted sleeves and 3 kg bronze bar. Different repair paths
consume the bar; the prediction is that repeated repairs exhaust locally
available bronze before day 90. Expected outcome: survival around 40 days,
with a `missing resource bronze-bar` block.

Expected output is frozen in [expected.txt](expected.txt).

## Limitations

Synthetic model, no joint optimisation, no spare procurement, no stochastic
timing, no calibrated degradation, and no physical safety implication.
