# S2 Joint Double-Failure — pre-registration

- Experiment ID: `s2-joint-double-failure`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-06 model-level joint inventory conflict.

## Question

Can an independent recovery envelope claim a double failure is recoverable
when the joint repair policy has only one shared consumable?

## Setup

Base-06 is derived from Base-05:

- `pump-p2` added; `potable-water` requires both pumps and the filter.
- Both pumps start available.
- One accepted sleeve remains; `bronze-bar` is 0.6 kg, below the 1.2 kg needed
  to machine another sleeve.

The independent 12 h envelope checks each pump separately against the same one
sleeve, so the pair `pump-p1,pump-p2` should look recoverable. A joint policy
repairs one pump, consumes the sleeve, and then cannot repair the second.

## Prediction

```text
independent_coverage=true
joint_survived_days=10
joint_blocked_by=missing resource bronze-bar: need 1.2, have 0.6
```

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

This is a deliberately constructed synthetic conflict to demonstrate why
independent coverage is an upper bound; it is not a physical claim.
