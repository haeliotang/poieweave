# N9 Resource Observation Capability Matrix

| Field | canonical | N5 FMU | N6 FMU |
| --- | --- | --- | --- |
| resource identity | NATIVE (RunningRepair.resources) | NATIVE (tNRobot) | NATIVE (tNRobot, toolTaskMask) |
| task identity | DERIVED from repair label/plans | NATIVE (t1..t3) | NATIVE (t1..t3) |
| selected resource | NATIVE selector + DERIVED mapping | NATIVE | NATIVE |
| alternative group | INPUT_ONLY | INPUT_ONLY (mask) | INPUT_ONLY (mask) |
| required tool | DERIVED (execution dependency) | NOT_AVAILABLE | NATIVE (toolTaskMask / toolAcquired) |
| upstream dependency | DERIVED (witness not exported) | NOT_AVAILABLE | INPUT_ONLY / generic failureKind3 |
| task start/end | DERIVED from RunningRepair/events | NATIVE | NATIVE |
| group makespan | NATIVE repair end / event | NATIVE | NATIVE |
| resource acquisition | NOT_OBSERVED | DERIVED from start | NATIVE toolAcquired; robot start derived |
| resource hold interval | CONTRACT_DERIVED group-level | group-derived | group-derived / toolReleased |
| resource release | NOT_OBSERVED (termination ≠ release) | NOT_OBSERVED | toolReleased field; group semantics |
| asset failure | NATIVE AssetFailure | failure flag/time | failure flag/time |
| repair interruption | NATIVE RepairInterrupted | interrupted flag | interrupted flag |
| restoration commit | NATIVE RepairCompleted + target snapshot | pumpRestored/latheRestored | restoredN |
| observation provenance | mixed; see contract | mixed | mixed |

## Explicit gaps

- canonical does not export a native resource-release event.
- canonical witness/execution_dependencies are now exported in `RepairObservation`
  as read-only observation fields (v5 migration); resource hold/release events remain
  contract-derived.
- N5 has no tool/upstream resource outputs.
- N6 upstream failure is generic (`failureKind=3`), not a named upstream asset.
- No FMU has step-level release.
