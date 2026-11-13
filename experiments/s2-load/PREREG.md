# S2 Site Load Sensitivity — pre-registration

- Experiment ID: `s2-load`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: constant-load study parameter on Base-05; model-level only.

## Question

At what point does a constant site load turn energy into the binding constraint,
rather than material stock?

## Method

Base-05, 90-day horizon, pump failures every 10 days plus a day-35 lathe
failure. Battery 200 kWh, charge cap 10 kW, PV average about 2.5 kW. Apply
constant loads 0/1/2/3/4 kW. Energy is netted per interval; if it would go
negative, the run stops at the estimated depletion time.

## Prediction

- Loads at or below ~2.5 kW should not change the 40-day material-limited result.
- 3–4 kW should deplete the battery early and make energy the limiting factor.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

Constant load is a study parameter; no demand profile, no load-dependent service
quality and no real device power model.
