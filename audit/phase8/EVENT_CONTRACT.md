# N8 Event Observation Contract

```text
event_sequence        strictly increasing, run-local, deterministic
event_id              ev-<seq:04>-<EventKind>
event_batch_sequence  batch that committed the event
at_s                  real simulation time, never offset for ordering
event_kind            InitialRepairStarted / RepairStarted / RepairCompleted /
                      AssetFailure / Resupply / RepairInterrupted /
                      EnergyDepleted / HorizonReached
event_origin          Exogenous / Endogenous
target_asset          asset/resource/label when applicable
event_status          Committed / Rejected / NotApplied
asset_statuses_before native current.assets snapshot immediately before
asset_statuses_after  native current.assets snapshot after commit
```

Event sequence is assigned by the engine's actual event path, not by observation
requests. `Unknown` is a model status; missing observation remains unobservable.
