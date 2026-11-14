# S2 Random Adversarial Sample — pre-registration

- Experiment ID: `s2-adversarial-random`
- Date: 2026-11-14
- Status: completed; 2/500 random draws reach 0 days
- Scope: 500 seeded random subset × failure-time draws.

## Question

How often does a random adverse draw reach a very short horizon compared with
the exhaustive subset search?

## Method

Base-08; five assets; 500 deterministic pseudo-random draws: random non-empty
subset and failure day in 0..90; 90-day horizon. Report zero/under-30/under-60
counts, mean, minimum, worst set and time.

Prediction: zero-day draws are rare because they require a specific set and an
early time.

## Limitation

Seeded sample, not exhaustive; five assets.
