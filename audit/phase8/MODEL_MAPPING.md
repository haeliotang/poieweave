# N8 Cross-Model Event Mapping (SCOPED)

## Shared observable contract

```text
PV failure:
  canonical  AssetFailure pv-1: Available -> Failed @ event time
  external   pvAvailable: 1 -> 0, failureAppliedTime_s
Repair completion:
  canonical  RepairCompleted pump-p1: Degraded -> Available
  external   repairCompletedTime_s
Repair interruption:
  canonical  RepairInterrupted; target not restored
  external   robotAvailable 1 -> 0, repairState=2, no repair completion
Energy depletion:
  canonical  EnergyDepleted event time
  external   depleted=1 at terminal time
```

## Enumerated accepted scenarios

- N8-01: PV failure 7200, repair completion 10800, energy depletion 53400
- N8-02: PV failure 4500 (step 1000, non-grid event), repair completion 10800
- N8-04: robot-m1 failure 7200, same-time asset failure + repair interruption

## Not compared

- selected resources / step-level lifetime (external FMU has no step-level resource lifecycle)
- exact repair-energy trajectory: canonical commits at completion, external draws uniformly;
  classified as `MODEL_ASSUMPTION_DIFFERENCE`, not numerical error.

This mapping is scoped to the enumerated N4 cases and does not establish general
trajectory-level cross-model validity.
