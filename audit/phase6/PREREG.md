# S5-V N6 — Contract-Driven Coordination & Causal Boundary Challenge (Preregistration)

Status: PREREGISTERED before N6 implementation.
Baseline: `c38ea8500426a5bc9ea9e4b521a9c4fcc0bf9423` (PR #576).

## First gate: Resource Lifetime Audit

The N5-03 scenario is re-examined before any new scheduling claim:

```text
lathe task 7200 s on robot-m1 (priority 1)
pump task 10800 s on robot-m2 (priority 2)
robot-m1 failure at selected boundary times
```

Boundary cases: 7199, 7200, 7201, 9000, 10800 s.

For each case record:

```text
resource_acquired_at, resource_released_at, step_completed_at,
group_committed_at, failure_applied_at, repair_interrupted,
restoration_committed, termination
```

The audit must distinguish:

- failure of a resource while its step is active;
- failure exactly when the step completes;
- failure after that step completed but before group commit;
- failure exactly at group commit;
- group-level versus step-level resource release.

No step-level resource-release claim is allowed unless the canonical engine and
the external model both expose and honor that lifetime.

## Required distinctions

N6 preserves:

```text
N5 INDEPENDENT RESOURCE SCHEDULING
  ESTABLISHED (SCOPED: fixed task contract)

N5 POST-INTERRUPTION REPLANNING
  NOT ESTABLISHED

N5 AUTONOMOUS RECOVERY
  NOT ESTABLISHED

N5 LEVEL 3 PLANNING INDEPENDENCE
  NOT ESTABLISHED
```

## Planned N6 experiments

- N6-A: N5-03 boundary failures and resource lifetime audit.
- N6-B: parameterized two-task scheduler (duration, priority, robot pool).
- N6-C: required tool failure and upstream `bus-1` failure.
- N6-D: same-time completion/failure and horizon boundary.
- N6-E: restoration state evidence (derived asset state + external restoration flags).

## Claims not made

- step-level resource release if canonical is group-level;
- post-interruption replanning;
- autonomous recovery;
- Level 3 independent task/plan generation;
- full S2 or physical correctness.
