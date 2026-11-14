# S3 Repair Deadline — pre-registration

- Experiment ID: `s3-repair-deadline`
- Date: 2026-11-14
- Status: completed; clock-advanced repair misses a deadline the instant model meets
- Scope: 3-day repair, failure day 10, deadline day 11.

## Question

Can repair-time semantics flip a deadline decision that is locally feasible?

## Method

Set the pump restorer duration to 3 days; failure at day 10; deadline at day 11.
Compare instantaneous completion (day 10) with clock-advanced completion
(day 13).

Prediction: instantaneous meets; clock-advanced misses.

## Limitation

Arithmetic completion model; one repair and one deadline.
