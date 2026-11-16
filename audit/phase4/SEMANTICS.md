# N4 Phase A — Canonical Event Semantics Audit

Source references are to the baseline `33583f5` engine.

## 1. Initial repair

`run_engine` computes initial unavailable assets after loading the model, builds
`pending` failures at `t=0`, calls `start_repair`, and stores a `RunningRepair`
with `end_s = now + schedule.makespan_s`. Repair energy is committed only when
the completion branch executes.

## 2. Failure during repair

At an event boundary, exogenous failures are applied to the asset model and
traced as `failure:<asset>`. If a failed asset is in the running repair's
`execution_dependencies` or `restoration_targets`, the engine terminates with
`Blocked("repair interrupted ...")`; the repair is not committed. Other
failures are queued as `pending` repairs.

## 3. PV failure

`apply_energy_interval_ledger` generates PV energy only while
`model.assets["pv-1"].status == Available`. A `failure:pv-1` event therefore
changes generation from the event boundary onward. Energy is conserved; there
is no instantaneous energy jump at the failure.

## 4. Restoration

There is no exogenous `AssetRestoration` event in `EngineOptions`. Restoration
is endogenous: a completed repair applies its plans to assets. A preset
restoration time cannot be represented as autonomous restoration.

## 5. Execution-resource failure

Tests `failure_of_the_selected_robot_interrupts_the_repair` and
`failure_of_an_upstream_dependency_interrupts_the_repair` establish that a
failed execution dependency or restoration target interrupts the repair and no
`repair_completed` is emitted.

## 6. Same-timestamp order

The engine checks repair completion before processing exogenous events at the
same timestamp. Trace order at `10800 s` in N4-03 is:

```text
repair_completed:pump-p1
failure:pv-1
```

## 7. Horizon boundary

`CompletedHorizon` is emitted when repair completes exactly at horizon
(`repair_completing_exactly_at_horizon_is_completed`). The engine checks the
horizon before processing a same-timestamp exogenous event after completion, so
a failure scheduled exactly at horizon is not processed as a causal event.
N4 does not use horizon-equal failures.

## 8. Load step

`EngineOptions.load_kw` is a constant. Canonical does **not** support an
arbitrary load-step event. N4-07 is therefore not implemented in this phase and
must not be claimed.

## Observation phases

`Before` is the state after the ledger has advanced to the event timestamp and
before completion/exogenous event processing. `After` is the committed state
after those transitions. Observations do not modify the canonical run; the
native noninterference tests compare the full outcome.

## Supported event types for N4

```text
AssetFailure (pv-1)        EXOGENOUS
RepairCompleted            ENDOGENOUS
EnergyDepleted             ENDOGENOUS TERMINATION
HorizonReached             ENDOGENOUS TERMINATION
Observation Before/After   OBSERVATION ONLY
```

`AssetRestoration`, `RepairInterrupted`, shared-resource scheduling and
`LoadChanged` are not part of the minimal N4-A/B set.
