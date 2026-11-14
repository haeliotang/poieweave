# S2 Time-Semantics Horizon — pre-registration

- Experiment ID: `s2-time-horizon`
- Date: 2026-11-14
- Status: completed; clock-advanced repair changes the horizon 6 vs 16 days
- Scope: 5-day repairs, 2-day pump failures, limited grease and no sleeves.

## Question

Can repair-duration clock advancement change the survival horizon, not just
event timestamps?

## Method

Base-08 modified: no sleeve stock, grease 0.15 kg, `replace-sleeve-p1` duration
5 days; pump failures every 2 days; 30-day horizon; compare instantaneous and
event-driven loops.

Prediction: event-driven pushes later failures past the horizon and survives
longer.

## Limitation

Modified scenario; one resource limit.
