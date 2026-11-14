# S3 Time Semantics — pre-registration

- Experiment ID: `s3-time-semantics`
- Date: 2026-11-14
- Status: completed; event-driven repair advances the clock
- Scope: dense failures inside a repair window and sparse 10-day failures.

## Question

Does advancing the clock by repair duration change when later events are
processed?

## Method

Base-08; dense failures at t=0/1 h and sparse 10-day failures; compare the
original instantaneous repair loop with the event-driven loop that advances
`now` by plan duration.

Prediction: instantaneous keeps event times 0/1 h; event-driven moves them to
3/6 h.

## Limitation

One scenario; no scheduler/preemption.
