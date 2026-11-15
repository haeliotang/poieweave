# S5 Migration Batch 2 — pre-registration

- Experiment ID: `s5-migration-batch2`
- Date: 2026-11-15
- Status: contention explained; dense now uses in-repair event timing and differs by design
- Scope: dense long repair and one- vs two-robot contention.

## Question

When migrating dense and resource-contended scenarios from legacy survival to the
canonical engine, are differences explainable by the new resource semantics?

## Method

Run both paths with identical inputs and record horizon, termination, event counts
and last committed event time.

Prediction: contention differences match robot serialization; dense failures during
repair take effect at their own time and may invalidate the repair.

## Limitation

Two scenarios; no resupply/load migration yet.
