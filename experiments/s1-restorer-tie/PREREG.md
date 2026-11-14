# S1 Restorer Tie-Break — pre-registration

- Experiment ID: `s1-restorer-tie`
- Date: 2026-11-14
- Status: completed; equal-duration tie is order-sensitive
- Scope: cheap vs expensive restorer with the same duration.

## Question

When two restorers have equal duration, does closure prefer the cheaper input
plan or the first one in the index?

## Method

Add a 3 h duplicate pump restorer that uses 0.20 kg grease instead of 0.05 kg;
analyze with cheap-first and expensive-first order; compare remaining grease.

Prediction: if cost-aware, both orders use the cheap restorer.

## Limitation

One tie pair; grease is the only differing input.
