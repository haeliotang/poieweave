# S3 Concurrency Deadline — pre-registration

- Experiment ID: `s3-concurrency-deadline`
- Date: 2026-11-14
- Status: completed; shared robot misses a deadline that separate robots meet
- Scope: two repairs, common 4 h deadline, one vs two robots.

## Question

Can each repair fit a 4 h deadline individually while the system still misses it
because of serialization or shared occupancy?

## Method

Base-08; pump (3 h) and lathe (2 h) both failed; common deadline 4 h; one-robot
and two-robot layouts.

Prediction: sequential misses; separate-robot parallel meets; shared-robot
parallel also misses.

## Limitation

Static makespan comparison, not a scheduler.
