# S2 Adversarial Subsets × Time Semantics — pre-registration

- Experiment ID: `s2-adversarial-time`
- Date: 2026-11-14
- Status: completed; clock-advanced repair removes the 0-day minimum
- Scope: all 31 failure subsets at days 0/10, 5-day repairs, 2-day failures.

## Question

Does clock-advanced repair time change the adversarial worst case over failure
subsets?

## Method

Base-08 with 5-day repairs, pump failures every 2 days, a day-35 lathe failure,
and every non-empty subset of five assets at day 0 or day 10; compare the minimum
instantaneous and event-driven horizons.

Prediction: repair advancement pushes the worst case above 0 days.

## Limitation

One long-repair setting; subset failures at t=0/t=10 only.
