# S2 Redundancy vs Shared Repair Robot — pre-registration

- Experiment ID: `s2-redundancy-robot`
- Date: 2026-11-14
- Status: completed; shared repair robot remains a blocker
- Scope: redundant pump plus a robot-m1 failure.

## Question

Does a redundant pump help when the shared repair robot fails?

## Method

Base-08; same redundant-pump configuration as `s2-redundancy`; add a robot-m1
failure at day 20. 10-day pump failures, 90 days.

Prediction: both modes eventually block on no restorer for robot-m1.

## Limitation

One repair asset; no robot redundancy.
