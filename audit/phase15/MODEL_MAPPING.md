# Model Mapping

| Canonical | N15 FMU |
|---|---|
| TaskCompleted event | `TaskCompleted` journal event |
| ResourceReleased event | `ResourceReleased` + `released_mask` |
| RunningRepair.execution_dependencies | per-unfinished-task dep mask |
| TaskResourceObservation | t1/t2 start/end/robot CSV fields |
| RepairCompleted | RepairCompleted + RestorationCommitted |

Canonical additionally releases named upstream dependencies at group completion;
the external N15 FMU tracks selected robot/tool lifetime only, so upstream release
is `NOT_COMPARABLE` and excluded from release-event equality.
