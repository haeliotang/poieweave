# S5 Migration Batch 3 — pre-registration

- Experiment ID: `s5-migration-batch3`
- Date: 2026-11-15
- Status: completed; load/resupply/battery boundaries checked
- Scope: load over repair, resupply before arrival, initial overcapacity.

## Question

Does canonical Batch 3 semantics agree with legacy on energy integration,
resupply timing and battery capacity validation?

## Method

Run canonical engine and the matching legacy entry point with identical inputs;
record horizon, final energy, and classification.

Prediction: load integrates identically; resupply before arrival is refused by
both; initial overcapacity is rejected by both.

## Limitation

Three scenarios; no joint repair+resupply+load ledger yet.
