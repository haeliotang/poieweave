# S3 Robot Count Sweep — pre-registration

- Experiment ID: `s3-robot-sweep`
- Date: 2026-11-14
- Status: completed; two robots are the minimum for the 4 h deadline
- Scope: three repairs (3 h, 2 h, 2 h), 1-3 robots.

## Question

How many robots are needed for three concurrent repairs to meet a common 4 h
deadline?

## Method

Greedy list-scheduling: assign each repair to the earliest-free robot; compute
makespan and whether it meets 4 h.

Prediction: one robot fails, two or three meet.

## Limitation

Synthetic durations; greedy schedule, not optimal search.
