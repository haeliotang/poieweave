# S2 Adversarial Subsets — pre-registration

- Experiment ID: `s2-adversarial-subsets`
- Date: 2026-11-14
- Status: completed; all 31 subsets searched, minimal 0-day set is filter-1
- Scope: every non-empty simultaneous subset of five assets at t=0 and day 10.

## Question

If every non-empty subset of five assets is failed at t=0, what is the minimal
set that reaches the minimum horizon?

## Method

Base-08; assets pump-p1, filter-1, robot-m1, pv-1, lathe-1; enumerate all 31
non-empty subsets at t=0 and at day 10; record the minimum horizon and the
smallest set reaching it for each time.

Prediction: a single shared component with no restorer.

## Limitation

Five assets, one failure time, t=0.
