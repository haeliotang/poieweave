# S3 Concurrent Scheduler — pre-registration

- Experiment ID: `s3-scheduler`
- Date: 2026-11-15
- Status: completed; 7/4/3 h makespans over 1/2/3 robots
- Scope: three repairs (3 h, 2 h, 2 h) on a shared robot pool.

## Question

Can a deterministic scheduler run repair plans concurrently over a robot pool?

## Method

Schedule three single-step plans over 1-3 robots; steps occupy any robot; each
plan remains sequential. Report makespan and plan end times.

Prediction: 7/4/3 h for 1/2/3 robots.

## Limitation

Greedy list scheduling; no preemption.
