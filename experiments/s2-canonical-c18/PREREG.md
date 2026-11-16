# S2 Canonical C18 Migration — pre-registration

- Experiment ID: `s2-canonical-c18`
- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Scope: 200 seeded random adversarial subsets, 30-day horizon, 5-day sleeve
  repair, grease 0.15 kg, sleeve 0.

## Question

How do zero-day rate, mean, and quantiles change under canonical semantics?

## Method

Reuse the original seed and mask generator (see SEED.md); compare legacy
instant, legacy event-driven, and canonical. Report zero count, mean and
p10/p50/p90.

Prediction: zero rate may agree, but mean/quantiles change because in-repair
failures interrupt.

## Limitation

One seed; deterministic generator only.
