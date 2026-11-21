# ADR — Explicit Repair Group

## Decision

Add `ExplicitRepairGroup` / `ExplicitRepairTask` and
`run_engine_with_repair_group[_event_observed]`. Tasks are turned into one
`Plan` each and scheduled by the existing canonical scheduler. The group
commits atomically through the existing post-repair `apply_plans` path.

## Classification

`EXECUTION_SEMANTIC_EXTENSION` — not observation-only.

## Non-goals

No step-level release, no replanning, no autonomous recovery, no alternate
execution loop. Legacy entry points remain `run_engine_inner(..., None)`.
