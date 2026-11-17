# N6 Phase A — Resource Lifetime Audit (source inspection)

Source: `crates/poie-viability/src/engine.rs`, `scheduler.rs`.

- `RunningRepair` holds one `end_s` = schedule makespan.
- `repair.resources` is the flattened selected resource list for all plans.
- `execution_dependencies` includes selected resources, required non-alternative
  resources, and witness dependencies.
- A failure of any dependency or restoration target immediately returns
  `Blocked("repair interrupted ...")`.
- Plans are applied only when the whole group reaches `end_s`; restoration is
  therefore group-committed, not per-task.
- There is no per-step resource release bookkeeping in `RunningRepair`.
- At the group completion timestamp, completion is processed before exogenous
  failure events.
- There is no post-interruption replanning path.

Consequence: canonical resource ownership is **group-level** for interruption
and restoration. A completed earlier step's resource is still treated as an
active dependency until group commit. N6-A must test this boundary and label
step-level lifetime as not represented, not verified.
