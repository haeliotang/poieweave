# S2 Survival Horizon — pre-registration

- Experiment ID: `s2-survival-horizon`
- Date: 2026-11-14
- Status: completed; final curve in [REPORT.md](REPORT.md)
- Scope: deterministic greedy policy over Base-05; model-level only.

## Question

How does the interval between repeated pump failures change the modelled
survival horizon of the combined `potable-water` + `workshop-operability`
services, given the same 90-day horizon, shared inventory and battery limits?

## Method

For pump-failure intervals 10, 15, 20 and 30 days, schedule pump failures from
`interval` through `< 90` days, plus one lathe failure at day 35. Run the
multi-service greedy recovery policy with the shared S1 closure, 200 kWh battery
capacity and 10 kW charge cap. The first blocked service ends the run.

## Prediction

The bronze-bar stock is the binding constraint. The horizon curve should rise
with the interval; at 10/15/20 days the run should stop at 40/60/80 days on
`missing resource bronze-bar`, while 30-day spacing should survive the 90-day
horizon.

Expected output is frozen in [expected.txt](expected.txt).

## Limitations

Synthetic stock, no operating demand, no joint optimisation, one lathe event,
no stochastic timing, no physical calibration.
