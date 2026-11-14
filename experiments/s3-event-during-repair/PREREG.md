# S3 Event During Repair — pre-registration

- Experiment ID: `s3-event-during-repair`
- Date: 2026-11-14
- Status: completed; the model has no mid-repair interruption
- Scope: a second failure one hour into a three-hour repair.

## Question

What happens when a new failure occurs inside the repair window?

## Method

Base-08; force pump-p1 failed; start its 3 h repair at t=0; schedule filter-1
failure at t=1 h; record receipts and blocker; compare event time with repair
duration.

Prediction: the event is inside the window, and the model processes it without
interrupting the applied repair.

## Limitation

Detects the gap; does not implement interruption.
