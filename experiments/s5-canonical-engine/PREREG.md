# S5 Canonical Engine — pre-registration

- Experiment ID: `s5-canonical-engine`
- Date: 2026-11-15
- Status: completed; one sparse scenario matches the legacy path
- Scope: Base-08 direct sleeve replacement, 5-day repair, failures on days 10/20.

## Question

Does the new canonical event kernel produce the same horizon as the legacy
event-driven path on a sparse scenario with direct replacement?

## Method

Run both paths with identical inputs; report canonical/legacy days, typed
termination and whether they agree.

Prediction: same 30-day horizon, `CompletedHorizon`.

## Limitation

One sparse scenario; dense/resource-contended scenarios may differ by design.
