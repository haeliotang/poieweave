# S2 Repair-Duration Sweep — pre-registration

- Experiment ID: `s2-time-repair-sweep`
- Date: 2026-11-15
- Status: completed; the time-semantics effect starts between 1 and 2 day repairs
- Scope: repair durations 0.125-5 days with 2-day failures.

## Question

At what repair duration does clock advancement start to change the horizon?

## Method

Base-08 modified (no sleeves, 0.15 kg grease); pump failures every 2 days;
repair 3 h / 12 h / 1 / 2 / 3 / 5 days; 30-day horizon; compare instantaneous
and event-driven horizons.

Prediction: no effect until repair approaches the failure interval, then a
monotone increase.

## Limitation

One failure cadence; one resource-limited scenario.
