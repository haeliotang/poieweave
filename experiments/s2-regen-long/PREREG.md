# S2 Regeneration × Long Repairs — pre-registration

- Experiment ID: `s2-regen-long`
- Date: 2026-11-14
- Status: completed; dense failures plus 5-day repairs change regeneration horizons
- Scope: Base-05/08, failure intervals 2/3/5/10 days.

## Question

When repairs are long and failures frequent, does clock-advanced repair time
change the regeneration horizon?

## Method

Set `replace-sleeve-p1` to 5 days; pump failures every 2/3/5/10 days plus a
day-35 lathe failure; compare instantaneous and event-driven horizons.

Prediction: at 2-5 day intervals, the event-driven loop pushes later failures
past the horizon and survives longer.

## Limitation

One long-repair setting; model-level units.
