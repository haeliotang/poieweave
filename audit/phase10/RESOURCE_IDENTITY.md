# N10 Resource Identity

Task-level validation requires:

```text
task_id
selected_robot
task_allowed_robot_mask
task_required_resources
actual_task_resources
```

Canonical source: native `RunningRepair.task_resources`, derived from the actual
schedule steps (`plan_index` + step resources). Group resources alone are not
sufficient evidence for per-task validity.
