# S5-V N8 — Event-Resolved Observatory & Cross-Model Trajectory Validation

Baseline: `dc9ebb39e6cf404ffd5dab880f99ec632700a306`.

## Scope

- Native event identity, strict deterministic sequence, batch identity.
- Same-timestamp completion/failure are distinct events.
- Event-level before/after native asset states.
- Time-observed vs event-observed vs direct outcome equality.
- N4/N5/N6 cross-model trajectory comparison (separate phase, not yet complete in this first slice).
- v3 frozen; v4 migration if runtime binding changes.

## Not claimed

- general trajectory-level cross-model validity
- full S2 cross-model validity
- step-level resource release
- event-engine priority input
- post-interruption replanning
- autonomous recovery
- Level 3 independent planning
- physical-world correctness
