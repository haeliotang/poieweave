# S3 Concurrency — pre-registration

- Experiment ID: `s3-concurrency`
- Date: 2026-11-14
- Status: completed; shared robot serializes two repairs
- Scope: two failed assets with shared vs separate robot occupancy.

## Question

Do two repairs that both occupy robot-m1 require serialization, while separate
robots allow parallel execution?

## Method

Base-08; pump and lathe both failed; both restorers occupy a robot. Layouts:
one robot (both use robot-m1) and two robots (lathe uses robot-m2). Compare
robot intervals, sequential makespan and ideal parallel makespan.

Prediction: one-robot overlaps, two-robots does not.

## Limitation

Static interval observation, not a concurrent scheduler; no deadline model.
