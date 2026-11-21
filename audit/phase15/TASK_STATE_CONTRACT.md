# Task State Contract

Task identity is `task_id` plus `plan_index`. Native execution emits:

```text
TaskCompleted(task_id, at_s)
ResourceReleased(resource_id, at_s)
```

Task completion is processed at the real task end boundary. Group completion is
processed only at the group makespan. If a failure hits an unfinished task's
active/future resource, the group interrupts and never restores.
