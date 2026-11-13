# S2 Resupply Sensitivity — pre-registration

- Experiment ID: `s2-resupply`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-06 model-level sensitivity; no physical claim.

## Question

Does one extra sleeve — a minimal external resupply — close the Base-06 joint
coverage gap identified in `s2-joint-coverage`?

## Method

Run the 0/1/2-failure joint coverage on Base-06. Then append one accepted sleeve
lot and rerun the same analysis. Compare independent coverage, joint coverage
and the optimistic gap.

## Prediction

Baseline: independent 25/56, joint 24/56, gap 1 (the pump pair). With one extra
sleeve, the pump pair should become jointly recoverable: joint 25/56 and gap 0.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

A single synthetic supply item; no delivery time, cost, or uncertainty. It
identifies causality, not feasibility.
