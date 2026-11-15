# S2 Canonical C6 Migration — pre-registration

- Experiment ID: `s2-canonical-c6`
- Date: 2026-11-16
- Status: completed; 24/24 joint agreement after fixing non-service failures
- Scope: Base-06 all-pairs potable-water coverage, 12 h horizon.

## Question

Does canonical engine reproduce independent 25/56 and joint 24/56, and if not,
what is the first divergence?

## Method

Enumerate the same failure cases as legacy `joint_coverage`; run canonical engine
for each recoverable case with failures at t=0. Compare joint sets.

Prediction: initial divergence likely on assets not required by the service; then
either classify as semantic or fix the executor.

## Limitation

Coverage adapter is external; engine itself does not enumerate subsets.
