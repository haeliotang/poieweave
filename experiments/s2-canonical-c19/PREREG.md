# S2 Canonical C19 Migration — pre-registration

- Experiment ID: `s2-canonical-c19`
- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Scope: 200 seeded random subsets with failure day 0–10, 30-day horizon,
  5-day sleeve repair, grease 0.15 kg, sleeve 0.

## Question

How do zero-day rate, mean and quantiles change when subset failures are delayed
to a random day?

## Method

Reuse the original seed and generator (SEED.md); compare legacy instant,
legacy event-driven and canonical. Report zero count, mean, p10/p50/p90.

Prediction: delayed failures still interact with 5-day repair; canonical shortens
the distribution.

## Limitation

One seed; deterministic generator only.
