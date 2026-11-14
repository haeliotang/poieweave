# S2 Time-Semantics Insert Boundary — pre-registration

- Experiment ID: `s2-time-insert`
- Date: 2026-11-14
- Status: completed; the insert boundary shifts from 6/8/10/12/12 to 15/20/26/31/31
- Scope: insert stock 1-6 with 5-day repairs and 10-day failures.

## Question

Does clock-advanced repair time shift the insert-stock boundary on the default
scenario?

## Method

Base-08 with `replace-sleeve-p1` duration 5 days; insert stock 1/2/3/4/6; pump
failures every 10 days; 90-day horizon; compare both time semantics.

Prediction: clock-advanced pushes failures later, so the same insert stock lasts
through a longer horizon.

## Limitation

Modified repair duration; one consumable.
