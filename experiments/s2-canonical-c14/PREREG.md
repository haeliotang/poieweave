# S2 Canonical C14 Migration — pre-registration

- Experiment ID: `s2-canonical-c14`
- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Scope: Base-08 dense pump failures every 2 days, 5-day sleeve repair, with and
  without a redundant pump.

## Question

Does redundancy preserve 90 days under dense failures, and what happens to the
baseline?

## Method

Compare legacy instant, legacy event-driven and canonical.

Prediction: redundant pump survives 90; baseline is interrupted by in-repair
failures at day 2 under canonical.

## Limitation

One redundancy layout.
