# ADR-0021R: Native Checkpoint/Resume via v11 Opt-in Extension

Status: ACCEPTED (user-authorized, Option 1).

## Context
N21-B was blocked because canonical interruption returns only
`EngineTermination::Blocked` plus trace/final_inventory. No native checkpoint
state exists; replay from JSONL or a new episode cannot prove continuity.

## Decision
Add a minimal opt-in v11 execution-semantic extension:
- `RepairCheckpoint` native state captured at `RepairInterrupted`.
- `run_engine_to_checkpoint(...)` preserves default `run_engine` behavior.
- `resume_engine_with_authorization(...)` validates a checkpoint-bound
  one-shot authorization and continues the same execution.
- New native events: CheckpointCreated, ReplanAuthorized, ReplanRejected,
  RepairResumed, TaskRestarted, RestorationCommitted.
- Default API and v10 behavior unchanged.

## Consequences
v10 certificates are frozen; affected claims receive v11 successors after
historical replay. No JSONL stitching, no episode reset, no multi-attempt.
