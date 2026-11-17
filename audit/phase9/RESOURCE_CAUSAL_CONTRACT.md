# N9 Resource-Causal Observation Contract (v1, preregistered)

Each event observation records:

```text
event_id
event_sequence
at_s
event_kind
event_origin
task_id
repair_group_id
resource_id
resource_role          ROBOT | TOOL | UPSTREAM_DEPENDENCY | RESTORATION_TARGET
asset_health_before    AVAILABLE | DEGRADED | FAILED | UNKNOWN
asset_health_after
allocation_state       AVAILABLE | ALLOCATED | HELD_BY_GROUP | RELEASED | NOT_OBSERVED
execution_dependency   true | false | NOT_OBSERVED
repair_group_state     RUNNING | COMPLETED | INTERRUPTED | NOT_OBSERVED
restoration_committed  true | false | NOT_OBSERVED
observation_source     NATIVE_CANONICAL_RUNTIME | NATIVE_EXTERNAL_FMU |
                       DERIVED_FROM_SCHEDULE | DERIVED_FROM_TRACE |
                       PREREGISTERED_INPUT | NOT_OBSERVED
observation_validity   OBSERVED | DERIVED | NOT_OBSERVED | NOT_COMPARABLE
```

Rules:

- Asset health, allocation, execution dependency, repair state and observation
  validity are separate dimensions.
- `UNKNOWN` is a model value; `NOT_OBSERVED` is the observation system's state.
- `RunTerminated` is not evidence of a resource release event.
- Group-level hold remains the canonical contract until a new model version
  explicitly proves step-level release.
