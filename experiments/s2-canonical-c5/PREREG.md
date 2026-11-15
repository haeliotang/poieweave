# S2 Canonical C5 Migration — pre-registration

- Experiment ID: `s2-canonical-c5`
- Date: 2026-11-16
- Status: completed; summary agreement on all four intervals
- Scope: Base-05 pump interval curve, 10/15/20/30 days, 90-day horizon.

## Question

Does the canonical engine reproduce C5's 40/60/80/90-day horizons and final
energy, and if the traces differ, where?

## Method

Run legacy `simulate_multi_service_longevity_with_limits` and canonical
`run_engine` with identical events, services, battery and horizon. Record horizon,
final energy, failure timestamps, event counts and first divergence.

Prediction: horizons/energy agree; canonical may record extra repair-completed
events.

## Limitation

Summary agreement only; trace event counts differ by completion records.
