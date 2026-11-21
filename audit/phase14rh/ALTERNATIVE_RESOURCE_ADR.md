# Alternative-Resource ADR

Current explicit task contract carries a concrete process/resource set; the
scheduler has per-task `ResourceRequirement::alternatives`, but
`ExplicitRepairTask` does not expose a task-specific robot OR group.

Status:

```text
TASK-SPECIFIC ROBOT OR + TOOL AND
  NOT REPRESENTABLE IN CURRENT V7 EXPLICIT CONTRACT
  ADDITIONAL EXECUTION-SEMANTIC EXTENSION REQUIRED
```

No v8 runtime change was made in this phase. N14RH-E remains NOT ESTABLISHED.

## Resolution

Implemented minimal v8 extension:

- `ExplicitRepairTask.robot_alternatives: Vec<String>`
- canonical scheduler reuses per-task `ResourceRequirement::alternatives`
- adapter passes task-specific OR groups; process occupies only non-OR resources
  (notably tool-t1)

OR cases:

```text
OR-01 same policy -> m1/m2 agreement
OR-03 different policy -> m1/m2 vs m2/m2 (both valid)
OR-04 failure m1 -> canonical interrupted, external completed
      classification VALID_POLICY_DEPENDENT_DIVERGENCE
OR-05 tool failure -> both interrupted
OR-06 illegal task resource mutation -> rejected
```
