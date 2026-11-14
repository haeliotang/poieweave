# S3 Per-Event Denial — pre-registration

- Experiment ID: `s3-denial`
- Date: 2026-11-14
- Status: completed; denial stops exactly at the matching event
- Scope: deny at day 5/10/20 after prior approvals.

## Question

Does a per-event denial block only the matching proposal while prior repairs
remain?

## Method

Base-08; pump failures at days 10 and 20; approved 8 h policy; deny at day
5/10/20 or no denial; 30-day horizon.

Prediction: day 10/20 denials stop at that day; a non-matching day has no effect.

## Limitation

One failure sequence; denial stops the run rather than continuing later events.
