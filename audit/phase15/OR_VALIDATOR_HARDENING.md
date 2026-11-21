# OR Validator Hardening (N15 Phase 0)

`audit/phase14rh/or_cases.py` now validates the full task contract:

- task id and selected robot against task-specific OR set;
- required tool AND in canonical resources and external toolTaskMask;
- task start/end vs external CSV;
- per-task illegal selection;
- same-robot overlap / resource contention;
- group makespan and group commit;
- failure target/time and per-model interruption sensitivity;
- premature restoration (any restoration before group makespan);
- policy-dependent divergence classification only when both selections are legal.

Negatives are executed through this comparator, not a standalone predicate:

```text
illegal-task-robot        RESOURCE_ELIGIBILITY_VIOLATION_A
missing-required-tool     REQUIRED_TOOL_MISSING_A
illegal-contention        RESOURCE_CONTENTION_VIOLATION
wrong-task-start          TASK_START_MISMATCH_A
premature-group-commit    PREMATURE_GROUP_COMMIT
both-missing-tool         REQUIRED_TOOL_MISSING_A + EXTERNAL_REQUIRED_TOOL_MISSING_A
```
