# Post-Interruption Representability Audit

## Canonical interruption result

On `AssetFailure` hitting an active dependency, canonical returns:

```rust
EngineTermination::Blocked(...)
EngineOutcome { termination, survived_s, trace, final_inventory }
```

The result does not expose the state required to continue the same execution:

```text
asset statuses at interruption      NOT EXPOSED
completed tasks                     NOT EXPOSED as durable state
unfinished tasks                    NOT EXPOSED
selected resources                  only inside textual trace/observation
active dependency closure           only inside observation
applied failure IDs                 NOT EXPOSED
group status / restoration target   NOT EXPOSED
energy/inventory state              only final inventory
```

`EngineTrace` is an audit log, not a resumable checkpoint. Reconstructing a
checkpoint from JSONL/trace would be a research adapter guess and cannot prove
native state continuity.

## Decision matrix

```text
R1 existing runtime continuation
  INSUFFICIENT: no durable checkpoint/state and no resume API.

R2 minimal checkpoint/resume extension
  REQUIRES canonical execution-semantic extension and v11 migration.
  Not implemented in N21.

R3 external orchestration with a new episode
  NOT NATIVE CONTINUITY: new model initialization rewrites the episode.
  Rejected as evidence for checkpoint/resume.
```

## Gate decision

```text
N21 POST-INTERRUPTION REPRESENTABILITY
  BLOCKED

N21 NATIVE ONE-SHOT REPLANNING
  NOT ESTABLISHED
```

No replanning execution was attempted. No events were assembled across
episodes. Canonical v10 is unchanged.

## What would be required

A v11 opt-in checkpoint/resume extension exposing, at minimum:

```text
execution_id, group_id, checkpoint_sequence, at_s
asset_statuses, inventory, energy_state
completed_tasks, unfinished_tasks
selected_resources, active_dependencies
applied_failure_ids, group_status, restoration_target
state_hash, predecessor_hash
```

plus native `CheckpointCreated`, `ReplanAuthorized`, `RepairResumed` events,
strict state continuity, one-shot authorization and atomic restoration.
