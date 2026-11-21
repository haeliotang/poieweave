# N14-R Representability Audit

## R1 — multiple plans from multiple failed assets

Existing `RunningRepair.plans: Vec<Plan>` and `schedule_plans_with_alternatives`
support multiple plans and parallel/serialized scheduling. Plans are derived one
per pending failure, so multiple failed assets could form multiple plans.

## R2 — multiple steps in one plan

`Plan.steps` are scheduled sequentially inside a plan and closure steps express
process prerequisites. This cannot represent two independent same-target tasks
with separate resources.

## R3 — explicit tasks, one restoration target

Not representable in v6: no API accepts task-level duration/resource contracts
independent of closure-derived plans. The explicit repair-group extension is
therefore required.

Decision:

```text
SAME-TARGET EXPLICIT TWO-TASK GROUP
  NOT REPRESENTABLE IN V6
  MINIMAL EXECUTION-SEMANTIC EXTENSION REQUIRED
```
