# S5 Semantics Parity Guard — pre-registration

- Experiment ID: `s5-semantics-parity`
- Date: 2026-11-15
- Status: completed; no advanced-horizon-shorter-than-instant violation
- Scope: three long-repair dense-failure scenarios.

## Question

Can the canonical clock-advanced semantics produce a horizon shorter than the
instantaneous legacy semantics in the tested scenarios?

## Method

For each scenario run both semantics with identical events, inventory and
battery, then require `advanced_days >= instant_days`.

Prediction: no violation in these scenarios. This is an observation, not a
universal invariant.

## Limitation

Three scenarios only; advanced horizon may be shorter in other models.
