# S3 Concurrency Certificate — pre-registration

- Experiment ID: `certificate-s3-concurrency`
- Date: 2026-11-14
- Status: completed; 11th bundle layer
- Scope: shared vs separate robot makespans and a common deadline.

## Question

Can concurrency/contention results be packaged as an independently checkable
certificate and included in the bundle?

## Method

Emit one-robot and two-robot rows with durations, occupancy overlap, sequential
and parallel makespans, and a common 4 h deadline. The Python verifier recomputes
sequential/parallel and deadline flags.

Prediction: `certificate=ok` and bundle verification remains green.

## Limitation

Static makespan model, no scheduler.
