# S2 Resupply × Time Semantics — pre-registration

- Experiment ID: `s2-resupply-time`
- Date: 2026-11-14
- Status: completed; both semantics give the same resupply-limited horizons
- Scope: delivery cadence 10/20/30/60 days with 5-day repairs and 2-day failures.

## Question

When resupply is active, does clock-advanced repair time change the horizon?

## Method

One insert initially; deliveries of one insert at the chosen cadence; 5-day
repairs; a pump failure every 2 days; 90-day horizon; compare instantaneous
resupply simulation with event-driven resupply simulation.

Prediction: the delivery cadence dominates and the horizons may coincide.

## Limitation

Insert-only consumable; energy omitted in the event-driven variant.
