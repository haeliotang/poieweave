# S5-V N5 — Independent Repair Coordination & Endogenous Recovery (Preregistration)

Status: PREREGISTERED before N5 FMU implementation.
Baseline: `41728951ef7e2f1e5beb6482f2e402b725bd29cb` (PR #575).

## Question

Can two independently implemented schedulers, receiving the same task/resource
contract but not each other's planning output, derive causally consistent
resource allocations, completion times, interruption decisions and
repair-driven restoration?

## Scope levels

- Level 1 execution independence: N4 already tested.
- **Level 2 scheduling independence: N5 target.**
- Level 3 planning independence: not targeted; both sides receive the same
  two-task set and fixed durations.

## Task contract

Synthetic base-08 variant:

```text
pump task:  target pump-p1,   process replace-sleeve-p1, duration 10800 s
lathe task: target lathe-1,   process replace-lathe-nut,  duration  7200 s
both require exactly one robot from {robot-m1, robot-m2}
robot pool supplied by the scenario
task priority: lathe first, pump second (explicit public policy)
scheduler rule: earliest available eligible robot; deterministic
```

Canonical initial state: pump-p1 degraded, lathe-1 failed. Both repairs are
pending at t=0 and are scheduled as one canonical repair group.

## Cases

| id | tasks | robots | failure | purpose |
| --- | --- | --- | --- | --- |
| N5-00 | pump | 1 | none | single-task control |
| N5-01 | lathe+pump | 1 | none | serialization on one robot |
| N5-02 | lathe+pump | 2 | none | legal parallel allocation |
| N5-03 | lathe+pump | 2 | selected robot-m1 at 7200 s | selected dependency failure |
| N5-04 | pump | 2 | unselected robot-m2 at 7200 s | unselected backup failure |

## Expected invariants

- no exclusive robot occupancy overlap;
- every task's chosen robot is from the explicit pool;
- task priority order preserved when serialized;
- selected resource failure interrupts and commits no restoration;
- unselected resource failure is recorded but does not interrupt;
- restoration only after all repair steps commit;
- no post-interruption replanning claim.

Expected canonical makespans:

```text
N5-00: 10800 s
N5-01: 18000 s
N5-02: 10800 s
N5-03: interrupted at 7200 s
N5-04: 10800 s, completed despite robot-m2 failure
```

## Allowed difference

Both sides may choose different but contract-valid robots. The comparator must
classify such choices as `DIFFERENT_BUT_VALID_DECISION`, not failure. Only
contract violations or unexplained completion/interruption differences fail.

## Negative controls

- overlapping robot occupancy;
- invoking a task before its robot is released;
- selected robot failure followed by successful repair;
- unselected robot failure causing interruption;
- restoration before completion;
- no restoration after completion;
- completion time displacement;
- reversed same-time order;
- evidence hash mutation.

## Claims not made

- Level 3 planning independence;
- post-interruption autonomous replanning;
- endogenous recovery after interruption;
- autonomous recovery;
- full S2 or physical correctness.
