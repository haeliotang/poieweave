# N9 Certificate Impact

Baseline v4 runtime: `8e4b7c2164d58e8294ddeb635ab2f08f47d68377`.

## Change

- `engine.rs`: observation-only extension of `RepairObservation` with
  `execution_dependencies`, `restoration_targets`, `witness`.
- No scheduler, event ordering, failure, interruption or restoration rule changed.
- No resource release event invented.

## Migration

- 41 v4 certificates frozen in `audit/certificates/v4-frozen.json`.
- v4 historical verifier at `8e4b7c21`.
- 41 v5 successors issued with relation `RESOURCE_OBSERVATION_ONLY_SUCCESSOR`.
- v5 binds engine/source environment, actual replay, and `audit/phase9/manifest.json`.
- v1/v2/v3/v4 historical verification retained.

## Not changed

- group-level resource lifetime
- step-level release
- event-engine priority
