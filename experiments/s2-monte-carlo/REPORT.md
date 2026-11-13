# S2 Monte Carlo Failure Schedules — report

- Date: 2026-11-14
- Status: model-level distribution; not a calibrated reliability forecast
- Runner: `cargo run --locked -q -p poie-viability --bin poie-monte-carlo`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
runs=100 min_days=24 median_days=72 mean_days=67.2 max_days=90 survived_90=39
bucket	[0,40)	15
bucket	[40,60)	22
bucket	[60,80)	15
bucket	[80,90)	9
bucket	[90,91)	39
```

## 2. Findings

- The single-interval baseline (40 days at 10-day spacing) is not representative:
  randomly spaced failures produce a broad distribution with median 72 days.
- 39/100 schedules survive the full 90-day window.
- The low tail (15 runs below 40 days) comes from short pump intervals that
  exhaust material/stock before day 40.
- The distribution is still governed by material closure, not battery energy.

## 3. Limits

- One deterministic LCG seed; this is a robustness illustration, not a
  statistical confidence interval.
- Failures are independent and schedule-only; repair times and repair failures
  are not modelled.
- No operating demand, water demand or wear.
- Synthetic model and quantities; no physical safety claim.

## 4. Next

1. Add a proper failure probability model and repeated seeds.
2. Add repair-duration and repair-failure distributions.
3. Couple demand/load to the schedule.
4. Report confidence intervals and compare against the interval curve.
