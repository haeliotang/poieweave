# S5-V N7 — Native Asset-State Observatory & Certificate v3 Migration

Baseline: `117ba8169995c83a0d4225925e3faf6025913a9e`.

## Scope

- Native AssetStatus snapshots read directly from canonical `current.assets`.
- Before/After observations at event boundaries.
- Noninterference: `run_engine` outcome equals `run_engine_observed` outcome.
- Repair-driven restoration state (pump-p1 completion).
- Certificate v3 migration for the 41 current canonical claim certificates.
- v1/v2 bytes remain frozen and historically verifiable.

## Not claimed

- step-level resource release
- event-engine priority input
- post-interruption replanning
- autonomous recovery
- Level 3 independent planning
- general trajectory-level or full S2 cross-model validity
- physical-world correctness

## Initial required scenarios

At minimum: no-failure control, native initial state, repair completion Before/After,
selected robot interruption Before/After, post-termination request, degraded initial state.
Other N7 scenario rows are scheduled as follow-up unless explicitly listed as tested.
