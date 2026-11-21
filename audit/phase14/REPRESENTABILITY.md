# N14 Representability Boundary

## Two-task canonical execution

The current canonical engine maintains one active `RunningRepair` at a time. Its
planner builds one plan per pending failure; the N14 two-task contract instead
describes two independent tasks in one repair group over the same group asset.
The N14 canonical adapter therefore explicitly requires exactly one task:

```rust
assert_eq!(tasks.len(), 1, "N14 canonical adapter supports exactly one task");
```

No canonical task/plan mapping exists for the N14 two-task contract without a
new execution-model version and certificate-impact migration. External two-task
evidence (N14-T00..T04) is real execution evidence, but is **not** cross-model
two-task validity.

## Consequence

- N14-A: ESTABLISHED for external 2-task / 4-failure execution.
- N14-B: NOT ESTABLISHED as written; the required two-task cross-model accepted
  scenario is `NOT_REPRESENTABLE`.
- N14-C: cross-model generated validation is scoped to one task, one robot,
  0-4 failures; 24 cases for each of two seeds.
