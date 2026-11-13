# S2 Self-Sustaining Metrics — pre-registration

- Experiment ID: `s2-metrics`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: formal metric definitions on Base-05/Base-08.

## Question

Can Survival Horizon, Recovery Coverage and Regenerative Capacity be defined as
explicit typed metrics and computed consistently?

## Method

Add `poie-viability::self_sustain` types:

- `SurvivalHorizonMetric`
- `RecoveryCoverageMetric`
- `RegenerativeCapacityMetric`

Compute on the 10-day pump-failure schedule and single-failure envelope.

Prediction: 40 / 60 days; 5/10 coverage; +20 days regenerative capacity.

## Limitation

Metrics are definitional and model-level; not physical forecasts.
