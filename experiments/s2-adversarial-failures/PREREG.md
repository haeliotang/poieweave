# S2 Adversarial Failure Search — pre-registration

- Experiment ID: `s2-adversarial-failures`
- Date: 2026-11-14
- Status: completed; a single filter failure is the minimal 0-day set
- Scope: all single and pair simultaneous failures over five assets.

## Question

Which minimal simultaneous failure set shortens survival the most?

## Method

Base-08; assets pump-p1, filter-1, robot-m1, pv-1, lathe-1; baseline and all
singles/pairs failed at t=0; 60-day horizon. Report horizon and blocker; select
the smallest set that reaches the minimum horizon.

Prediction: shared components without restorers dominate.

## Limitation

Five assets, singles/pairs only, t=0 failures.
