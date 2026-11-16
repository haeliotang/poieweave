# S2 Canonical C12 Migration — pre-registration

- Experiment ID: `s2-canonical-c12`
- Date: 2026-11-16
- Status: SUPERSEDED — SEMANTIC_CORRECTION
- Scope: Base-08 grease boundary, 5-day repairs, pump failure every 2 days.

## Question

Does canonical repair-time semantics reproduce the event-driven 10/16/21-day
grease boundary?

## Method

Run legacy instantaneous, legacy event-driven, and canonical with identical
inputs; compare days/final energy/termination.

Prediction: canonical applies event-time failure during a 5-day repair and
interrupts at day 2; the old boundary is superseded.

## Limitation

Only the grease component of C12; insert/horizon components remain.
