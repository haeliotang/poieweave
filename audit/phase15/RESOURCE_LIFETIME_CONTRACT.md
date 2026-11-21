# Task-Level Resource Lifetime Contract

Modes:

```text
GROUP_HOLD   default v8 semantics, unchanged
TASK_RELEASE opt-in explicit repair group mode
```

Task states: PENDING, RUNNING, COMPLETED, INTERRUPTED.

Resource sets are computed from all unfinished tasks:

- ACTIVE: resource used by a task with start <= now < end;
- RESERVED_FOR_FUTURE: selected for a task starting later;
- RELEASED: no unfinished task uses the resource and it was held by the group.

A completed task never releases a resource still used by another unfinished
task. Group restoration remains atomic after all tasks complete.
