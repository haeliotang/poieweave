# S2 Canonical C13 Migration — pre-registration

- Experiment ID: `s2-canonical-c13`
- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Scope: Base-08 load 2.6/3.0/4.0 kW, 5-day repair, failures every 10 days.

## Question

Are energy boundaries invariant under canonical repair-time semantics, and do
exact depletion times agree?

## Method

Compare legacy instant, legacy event-driven and canonical days/energy.

Prediction: day-level boundaries agree; canonical may shift exact depletion by
scheduling the full repair chain.

## Limitation

Three loads; model-level.
