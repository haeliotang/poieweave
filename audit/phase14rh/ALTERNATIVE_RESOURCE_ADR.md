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
