# N7 Asset-State Observation Contract

```text
observation_id     = "<at_s>/<phase>/<asset_id>"
at_s               = simulation seconds
phase              = Before | After
observation_status = OBSERVED | UNOBSERVABLE_AFTER_TERMINATION | UNOBSERVABLE_NOT_MATERIALIZED
asset_id           = canonical runtime model asset id
asset_status       = Available | Degraded | Failed | Unknown
status_source      = NATIVE_CANONICAL_RUNTIME | UNOBSERVABLE
running_repair     = native RunningRepair snapshot or null
termination        = full-run termination context
```

`Unknown` is a model status. `UNOBSERVABLE*` is an observation-system state and is
never mapped to `Unknown`.

Before = after ledger advance to the timestamp and before completion/exogenous
event processing. After = after the event batch at that timestamp (or after a
sample-only boundary). For same-timestamp completion+failure, After is the batch
state; N7 does not invent a per-event phase beyond these two.

AssetStatus is not Capability Availability, Execution Dependency, or Repair State.
