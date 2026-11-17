# N5 Phase A — Canonical Scheduling and Recovery Audit

## start_repair

`start_repair(current, pending, inventory, now, remaining, resources,
alternative_pool)` analyzes every pending failure into a `Plan`, then calls
`schedule_plans_with_alternatives`. The resulting schedule is stored in one
`RunningRepair`; all plans share one `end_s` (makespan).

## Resource choice

`schedule_plans_with_alternatives` builds tasks in plan/step order. Required
resources are process `occupies` minus replaceable alternatives; explicit
alternative groups are OR. The scheduler assigns earliest available resources.
Selected `step.resources` are added to `execution_dependencies`; unselected
alternatives are not.

## Selected vs unselected failure

A failure of any **selected** resource in `execution_dependencies` or a
restoration target returns `Blocked("repair interrupted ...")`. A failure of an
unselected alternative is not in `execution_dependencies` and is traced but does
not interrupt (confirmed by N5-04 probe).

## Completion / restoration

Plans are applied only when the whole `RunningRepair` reaches `end_s`; restoration
targets are committed at that point. There is no per-task early restoration and
no post-interruption replanning in the current engine.

## Concurrency

Multiple tasks can run concurrently only if the scheduler assigns disjoint
resources. The canonical trace exposes the repair group's plans, resources and
completion time; step-level schedule evidence for N5 is produced by a separate
read-only probe using the public scheduler API with the same plans/pool.

## Recovery

Only **Repair-Driven Restoration** exists: assets restored by completed plans.
There is no exogenous restoration event, no post-interruption rerun, and no
autonomous recovery. Post-interruption replanning is NOT ESTABLISHED.

## Load step

`EngineOptions.load_kw` remains constant; load-step events are not representable.

## Capability matrix

| capability | canonical | external N5 | status |
| --- | --- | --- | --- |
| single task, one robot | yes | target | SUPPORTED_BY_BOTH_TARGET |
| two tasks, one robot serial | yes | target | SUPPORTED_BY_BOTH_TARGET |
| two tasks, two robots parallel | yes | target | SUPPORTED_BY_BOTH_TARGET |
| selected robot failure interrupts | yes | target | SUPPORTED_BY_BOTH_TARGET |
| unselected robot failure ignored | yes | target | SUPPORTED_BY_BOTH_TARGET |
| per-task independent completion/restore | no (group commit) | group commit | NOT_REPRESENTABLE |
| post-interruption replanning | no | no | NOT_ESTABLISHED |
| autonomous recovery | no | no | NOT_ESTABLISHED |
| load step | no | no | NOT_REPRESENTABLE |
