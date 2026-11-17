# N8 Observation Semantics

Canonical processes a completion and a same-timestamp exogenous failure in separate
loop batches. Therefore at `t=10800`:

```text
event 2  RepairCompleted  batch 2  pump-p1 Degraded -> Available
event 3  AssetFailure     batch 3  robot-m1 Available -> Failed
```

Both carry `at_s=10800`; ordering is represented only by `event_sequence` and
`event_batch_sequence`. No `+0.001` offset is used.

Time-based `ObservationPhase::Before/After` remains backward compatible. Event
observations use `run_engine_event_observed`, which records committed events and
before/after states automatically.
