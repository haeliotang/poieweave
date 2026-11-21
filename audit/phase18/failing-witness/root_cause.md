# N18 OR + Split Root Cause

Minimal input: Task B (robot-m1) scheduled first, Task A (robot OR m1/m2)
scheduled second and actually selects robot-m2 under split profile
(robot-m1 -> bus-1, robot-m2 -> bus-2).

Observed at TaskCompleted(A) @7200:

```text
execution_dependencies = [bat-1, bus-1, bus-2, charger-1, robot-m1]
```

`bus-2` is Task A's upstream dependency and should leave the closure when A
completes, because Task B only uses robot-m1/bus-1. Instead `bus-2` remains and
is only released at group completion.

Root cause: `start_explicit_repair` builds per-task `task_dependencies` from
`process.occupies` via static plans. For tasks with `robot_alternatives`, the
adapter leaves `process.occupies` empty and the scheduler assigns the actual
resource in `schedule.steps`. The task dependency closure is therefore computed
from an empty/static resource set, so upstream assets of the actual selected
robot are not registered as task dependencies and cannot be removed at task
completion.

Classification: INCORRECT_TASK_DEPENDENCY_CONSTRUCTION.
