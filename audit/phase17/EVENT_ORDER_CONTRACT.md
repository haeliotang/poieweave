# Event Order Contract

Same-time priority:

```text
TaskCompleted
ResourceReleased
DependencyDeactivated
RepairCompleted (only at makespan)
AssetFailure
RepairInterrupted
RestorationCommitted
```

Canonical and external event trajectories are compared in original order within
their own sequence. Same-time event sets must agree; differing order is allowed
only inside an unconstrained class such as multiple DependencyDeactivated events.
