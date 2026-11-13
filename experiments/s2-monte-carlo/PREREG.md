# S2 Monte Carlo Failure Schedules — pre-registration

- Experiment ID: `s2-monte-carlo`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: deterministic pseudo-random schedules over Base-05; model-level only.

## Question

Under randomly spaced pump and lathe failures, what does the 90-day survival
distribution look like, rather than a single hand-picked interval?

## Method

20 seeds × 100 runs (2000 runs) with fixed LCG seeds:

- pump-failure interval uniform in 5..=30 days, repeated until day 90;
- one lathe failure uniformly in 20..=60 days;
- shared inventory, multi-service greedy recovery;
- 200 kWh battery / 10 kW charge cap;
- 90-day horizon.

Report min, p10, median, mean, p90, max and 90-day survival count, plus
histogram buckets.

## Expected distribution

The earlier interval study suggests a median around 60–75 days and broad
dispersion; more frequent pump failures should dominate the low tail. This is a
distributional robustness check, not a calibrated reliability forecast.

## Limitation

The generator is pseudo-random but deterministic; no failure probability model,
no repair delay distribution, no operating demand.
