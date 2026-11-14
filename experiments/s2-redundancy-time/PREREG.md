# S2 Redundancy × Time Semantics — pre-registration

- Experiment ID: `s2-redundancy-time`
- Date: 2026-11-15
- Status: completed; redundancy dominates, both semantics reach 90 days
- Scope: baseline vs redundant pump, 5-day repairs, 2-day failures.

## Question

Under dense failures and long repairs, does a redundant pump remove the
time-semantics difference?

## Method

Base-08 baseline vs redundant-pump (pump-p1/pump-p2 alternative service support);
5-day repair; pump failures every 2 days plus day-35 lathe; 90-day horizon;
compare instantaneous and event-driven.

Prediction: redundancy restores the full horizon under both semantics.

## Limitation

One redundancy layout; model-level.
