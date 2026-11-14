# S2 Redundant Pump — pre-registration

- Experiment ID: `s2-redundancy`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: a second pump and service-level requires_any.

## Question

Does a redundant pump extend the potable-water horizon under repeated pump
failures?

## Method

Base-08; add pump-p2 available; move pump-p1 into potable-water requires_any
together with pump-p2. 10-day pump failures + lathe failure, 90 days.

Prediction: redundancy removes the pump bottleneck and may expose another.

## Limitation

Service-level N+1 only; no shared plumbing or control dependency.
