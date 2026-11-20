# Before/After Semantics

Captured at the actual transition site:

- `InitialRepairStarted`: robot selected 0 -> n; dependency mask 0 -> active
- `AssetFailure`: failure_applied 0 -> 1
- `RepairInterrupted`: repair_state Running(0) -> Interrupted(2), after failure state
- `RepairCompleted`: repair_state Running(0) -> Completed(1)
- `RestorationCommitted`: restoration_state 0 -> 1
- `RunTerminated`: termination_state 0 -> 1

Same-timestamp events retain separate seq and independent before/after.
Canonical before/after is currently `NOT_OBSERVED` in the N11 evidence; hence
N11-C is not established.
